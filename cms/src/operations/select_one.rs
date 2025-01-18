#![allow(unused)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use case::CaseExt;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::{sqlite::SqliteRow, Row};
use std::{future::Future, marker::PhantomData};

use queries_for_sqlx::{
    ident_safety::{define_schema, PanicOnUnsafe},
    prelude::*,
    quick_query::QuickQuery,
};
use sqlx::{Database, Pool, Sqlite};

use crate::{
    build_tuple::BuildTuple,
    dynamic_schema::{
        DynGetOneWorker, DynamicRelationResult, COLLECTIONS,
        RELATIONS,
    },
    error::{self, GlobalError},
    filters::{ById, Filters},
    queries_bridge::SelectSt,
    relations::{relation, LinkData, Relation},
    traits::Collection,
    tuple_index::{tuple_as_map::TupleElementKey, TupleAsMap},
};

pub trait GetOneWorker: Sync + Send {
    type Inner: Default + Send + Sync;
    type Output;
    fn on_select(
        &self,
        data: &mut Self::Inner,
        st: &mut SelectSt<Sqlite>,
    ) {
    }
    fn from_row(&self, data: &mut Self::Inner, row: &SqliteRow) {
    }
    fn sub_op<'this>(
        &'this self,
        data: &'this mut Self::Inner,
        pool: Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + 'this {
        async {}
    }
    fn take(self, data: Self::Inner) -> Self::Output;
}

pub struct GetOneOp<C, L, F> {
    links: L,
    queries: F,
    _pd: PhantomData<(C, L, F)>,
}

pub fn get_one<C>() -> GetOneOp<C, (), ()> {
    GetOneOp {
        links: (),
        queries: (),
        _pd: PhantomData,
    }
}

impl<Base, L, Q> GetOneOp<Base, L, Q>
where
    L: BuildTuple,
    Q: BuildTuple,
{
    pub fn query<N>(self, ty: N) -> GetOneOp<N, L, Q::Bigger<N>>
    where
        N: Filters<Base>,
    {
        GetOneOp {
            links: self.links,
            queries: self.queries.into_bigger(ty),
            _pd: PhantomData,
        }
    }
    pub fn by_id(
        self,
        id: i64,
    ) -> GetOneOp<Base, L, Q::Bigger<ById>> {
        GetOneOp {
            links: self.links,
            queries: self.queries.into_bigger(ById(id)),
            _pd: PhantomData,
        }
    }
    pub fn link_data<N>(
        self,
        ty: N,
    ) -> GetOneOp<Base, L::Bigger<N::Worker>, Q>
    where
        N: LinkData<Base, Worker: GetOneWorker + Send>,
    {
        GetOneOp {
            links: self.links.into_bigger(ty.init()),
            queries: self.queries,
            _pd: PhantomData,
        }
    }
    pub fn relation<ToCat>(
        self,
    ) -> GetOneOp<
        Base,
        L::Bigger<<Relation<ToCat> as LinkData<Base>>::Worker>,
        Q,
    >
    where
        Relation<ToCat>:
            LinkData<Base, Worker: GetOneWorker + Send>,
    {
        GetOneOp {
            links: self
                .links
                .into_bigger(Relation(PhantomData).init()),
            queries: self.queries,
            _pd: PhantomData,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GetOneOutput<C, D> {
    pub id: i64,
    pub attr: C,
    pub links: D,
}

impl<C, R, Q> GetOneOp<C, R, Q>
where
    C: Collection<Sqlite>,
    R: GetOneWorker + Send + Sync,
    Q: Filters<C>,
{
    pub async fn exec_op(
        mut self,
        db: Pool<Sqlite>,
    ) -> Option<GetOneOutput<C, TupleAsMap<R::Output>>> {
        let mut st =
            stmt::SelectSt::init(C::table_name().to_string());

        st.select_aliased(
            C::table_name().to_string(),
            "id".to_string(),
            "local_id",
        );

        C::on_select(&mut st);
        self.queries.on_select(&mut st);

        let mut worker_data = R::Inner::default();

        self.links.on_select(&mut worker_data, &mut st);

        let res = st
            .fetch_optional(&db, |r| {
                let id: i64 = r.get("local_id");
                let attr = C::from_row_noscope(&r);
                self.links.from_row(&mut worker_data, &r);
                Ok(GetOneOutput {
                    id,
                    attr,
                    links: (),
                })
            })
            .await
            .unwrap()?;

        self.links.sub_op(&mut worker_data, db).await;
        let data = self.links.take(worker_data);

        return Some(GetOneOutput {
            id: res.id,
            attr: res.attr,
            links: TupleAsMap(data),
        });
    }
}

#[derive(Debug, Deserialize)]
pub struct InputGetOne {
    pub filters: Map<String, Value>,
    pub relations: Map<String, Value>,
    pub id: i64,
}

#[derive(Debug, Serialize)]
pub struct GetOneOuputDynamic {
    pub id: i64,
    pub attr: Value,
    pub relations: Map<String, Value>,
}

#[axum::debug_handler]
pub async fn get_one_dynamic(
    db: State<Pool<Sqlite>>,
    collection_name: Path<String>,
    input: Json<InputGetOne>,
) -> Result<Json<GetOneOuputDynamic>, GlobalError> {
    let collection_gaurd = COLLECTIONS.read().await;
    let relation_gaurd = RELATIONS.read().await;

    let collection = collection_gaurd
        .get(&collection_name.0.to_camel())
        .ok_or(error::entry_not_found(&collection_name.0))?;

    let (mut rels, tra) = {
        let mut rels = vec![];
        let mut trans = vec![];

        // key: snake_case
        'found: for (key, value) in input.0.relations.iter() {
            for r in relation_gaurd
                .get(collection.table_name())
                .unwrap()
            {
                match r
                    .clone()
                    .init_on_get(&key, value.clone())
                {
                    DynamicRelationResult::Ok(ok) => {
                        rels.push(ok);
                        trans.push(key.to_owned());
                        continue 'found;
                    }
                    DynamicRelationResult::InvalidInput(err) => {
                        return Err(format!(
                            "relation {key} invalid input for {}: {}",
                            collection.table_name(),
                            err
                        ))?
                    }
                    DynamicRelationResult::NotFound => {}
                }
            }

            return Err(error::to_refactor(&format!(
                "relation {key} not found for {}",
                collection.table_name()
            ))
            .into());
        }
        (rels, trans)
    };

    let mut st = stmt::SelectSt::init(
        collection.table_name().to_string(),
    );

    for rel in rels.iter_mut() {
        rel.on_select(&mut st);
    }

    collection.on_select(&mut st);

    st.select_aliased(
        collection.table_name().to_string(),
        "id".to_string(),
        "local_id",
    );

    let id = input.0.id;

    st.where_(col("local_id").eq(id));

    let mut res = st
        .fetch_optional(&db.0, |r| {
            let attr = collection.from_row_scoped(&r);
            let id: i64 = r.get("local_id");

            for rel in rels.iter_mut() {
                rel.from_row(&r)
            }

            return Ok(GetOneOuputDynamic {
                id,
                attr,
                relations: Map::default(),
            });
        })
        .await
        .unwrap()
        .ok_or(error::to_refactor("entry with id not found"))?;

    // todo: concurent awaits

    for mut rel in rels.iter_mut() {
        rel.sub_op(db.0.clone()).await;
    }
    for (mut rel, t) in rels.into_iter().zip(tra) {
        let taken = rel.take();
        res.relations.insert(t, taken);
    }

    // hold the gaurds, to not change the schema while holding old schema objects
    drop(collection_gaurd);
    drop(relation_gaurd);

    Ok(Json(res))
}
