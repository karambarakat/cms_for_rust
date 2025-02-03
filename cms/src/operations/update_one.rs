use sqlx::Row;
use std::{future::Future, marker::PhantomData};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use case::CaseExt;
use queries_for_sqlx::{
    ident_safety::PanicOnUnsafe, prelude::*,
    quick_query::QuickQuery,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::{sqlite::SqliteRow, Pool, Sqlite};

use crate::{
    build_tuple::BuildTuple, dynamic_schema::{
        DynamicRelationResult, ValidatedAndTyped, COLLECTIONS,
        RELATIONS,
    }, error::{self, insert::InsertError, GlobalError}, filters::ById, queries_bridge::UpdateSt, relations::{
        many_to_many::ManyToMany,
        optional_to_many::OptionalToMany, LinkData, LinkId,
        LinkSpecCanInsert, Linked, Relation, UpdateId,
    }, traits::Resource, tuple_index::TupleAsMap
};

pub trait UpdateOneWorker: Sync + Send {
    type Inner: Default + Send + Sync;
    type Output;
    fn on_update(
        &self,
        data: &mut Self::Inner,
        st: &mut UpdateSt<Sqlite>,
    ) {
    }
    fn from_row(&self, data: &mut Self::Inner, row: &SqliteRow) {
    }
    fn sub_op2<'this>(
        &'this self,
        data: &'this mut Self::Inner,
        pool: Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + 'this {
        async {}
    }
    fn sub_op1<'this>(
        &'this self,
        data: &'this mut Self::Inner,
        pool: Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + 'this {
        async {}
    }
    fn take(self, data: Self::Inner) -> Self::Output;
}

pub struct UpdateOneOp<C, L> {
    links: L,
    input: C,
}

pub fn update_one<C: Resource<Sqlite>>(
    input: C::PartailCollection,
) -> UpdateOneOp<C::PartailCollection, ()> {
    UpdateOneOp { links: (), input }
}

use super::select_one::GetOneOutput;

impl<Base, L> UpdateOneOp<Base, L>
where
    L: BuildTuple,
{
    pub fn link_data<N>(
        self,
        ty: N,
    ) -> UpdateOneOp<Base, L::Bigger<N::Worker>>
    where
        N: LinkData<Base>,
        <N as LinkData<Base>>::Worker: UpdateOneWorker + Send,
    {
        UpdateOneOp {
            links: self.links.into_bigger(ty.init()),
            input: self.input,
        }
    }
    pub fn update_id<
        N,
        /// this is what the spec require the input to be (mostly infered)
        I,
    >(
        self,
        // what the spec require the input to be
        id: I,
    ) -> UpdateOneOp<
        Base,
        L::Bigger<
            // what the link says the worker should be
            <UpdateId<I, N> as LinkData<Base>>::Worker,
        >,
    >
    where
        Base: Linked<N, Spec: LinkSpecCanInsert>,
        UpdateId<I, N>: LinkData<Base, Worker: UpdateOneWorker>,
    {
        UpdateOneOp {
            links: self.links.into_bigger(
                UpdateId {
                    id,
                    _pd: PhantomData,
                }
                .init(),
            ),
            input: self.input,
        }
    }
}

impl<Base, Workers> UpdateOneOp<Base, Workers>
where
    Workers: UpdateOneWorker,
{
    pub async fn exec_op(
        self,
        db: Pool<Sqlite>,
    ) -> Result<
        GetOneOutput<Base, TupleAsMap<Workers::Output>>,
        InsertError,
    > {
        todo!("use update_one_dynamic instead")
    }
}

#[derive(Debug, Deserialize)]
pub struct InputUpdatetOne {
    pub id: i64,
    pub partial: Value,
    pub relations: Map<String, Value>,
}

#[derive(Debug, Serialize)]
pub struct OuputDynamic {
    id: i64,
    attr: Value,
    relations: Map<String, Value>,
}

#[axum::debug_handler]
pub async fn update_one_dynmaic(
    db: State<Pool<Sqlite>>,
    collection_name: Path<String>,
    input: Json<InputUpdatetOne>,
) -> Result<Json<OuputDynamic>, GlobalError> {
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
                    .init_on_update(&key, value.clone())
                {
                    DynamicRelationResult::Ok(ok) => {
                        rels.push(ok);
                        trans.push(key.clone());
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

    let mut st =
        stmt::UpdateSt::init(collection.table_name().to_owned());

    let id = input.id;

    st.where_(col("id").eq(id));

    collection.on_update(input.0.partial, &mut st)?;

    for rel in rels.iter_mut() {
        rel.sub_op1(db.0.clone()).await;
    }

    for rel in rels.iter_mut() {
        rel.on_update(&mut st);
    }

    if st.set_len() == 0 {
        Err(String::from("no-op"))?;
    }

    let mut res = st
        .returning_(vec!["*"])
        .fetch_one(&db.0, |r| {
            let c = collection.from_row_noscope(&r);
            for rel in rels.iter_mut() {
                rel.from_row(&r);
            }
            Ok(OuputDynamic {
                id: r.get("id"),
                attr: c,
                relations: Default::default(),
            })
        })
        .await
        .unwrap();

    for rel in rels.iter_mut() {
        rel.sub_op2(db.0.clone()).await;
    }

    for (mut rel, tra) in rels.into_iter().zip(tra.into_iter()) {
        let value = rel.take();
        res.relations.insert(tra, value);
    }

    Ok(Json(res))
}
