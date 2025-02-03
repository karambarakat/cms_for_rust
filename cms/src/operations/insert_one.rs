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
    build_tuple::BuildTuple,
    dynamic_schema::{
        DynamicRelationResult, ValidatedAndTyped, COLLECTIONS,
        RELATIONS,
    },
    error::{self, insert::InsertError},
    filters::ById,
    queries_bridge::InsertSt,
    relations::{
        many_to_many::ManyToMany,
        optional_to_many::OptionalToMany, LinkData, LinkId,
        LinkSpecCanInsert, Linked, Relation,
    },
    traits::Resource,
    tuple_index::TupleAsMap,
};

pub trait InsertOneWorker: Sync + Send {
    type Inner: Default + Send + Sync;
    type Output;
    fn on_insert(
        &self,
        data: &mut Self::Inner,
        st: &mut InsertSt<Sqlite>,
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

pub struct InsertOneOp<C, L> {
    links: L,
    input: C,
}

pub fn insert_one<C>(input: C) -> InsertOneOp<C, ()> {
    InsertOneOp { links: (), input }
}

// pub use does_these_belong_to_this_mod::*;

use super::select_one::GetOneOutput;
// mod does_these_belong_to_this_mod {
//     use std::{future::Future, marker::PhantomData};
//
//     use queries_for_sqlx::{
//         prelude::{col, stmt, ExecuteNoCache},
//         InitStatement,
//     };
//     use sqlx::{sqlite::SqliteRow, Pool, Sqlite};
//
//     use crate::orm::{
//         operations::{IdOutput, SimpleOutput},
//         relations::{
//             many_to_many::ManyToMany,
//             optional_to_many::OptionalToMany, LinkData,
//             LinkIdWorker, LinkSpecCanInsert, Linked,
//         },
//         Collection,
//     };
//
//     use super::InsertOneWorker;
//
//
// }

impl<Base, L> InsertOneOp<Base, L>
where
    L: BuildTuple,
{
    pub fn link_data<N>(
        self,
        ty: N,
    ) -> InsertOneOp<Base, L::Bigger<N::Worker>>
    where
        N: LinkData<Base>,
        <N as LinkData<Base>>::Worker: InsertOneWorker + Send,
    {
        InsertOneOp {
            links: self.links.into_bigger(ty.init()),
            input: self.input,
        }
    }
    pub fn link_id<
        N,
        /// this is what the spec require the input to be (mostly infered)
        I,
    >(
        self,
        // what the spec require the input to be
        id: I,
    ) -> InsertOneOp<
        Base,
        L::Bigger<
            // what the link says the worker should be
            <LinkId<I, N> as LinkData<Base>>::Worker,
        >,
    >
    where
        Base: Linked<N, Spec: LinkSpecCanInsert>,
        LinkId<I, N>: LinkData<Base, Worker: InsertOneWorker>,
    {
        InsertOneOp {
            links: self.links.into_bigger(
                LinkId {
                    id,
                    _pd: PhantomData,
                }
                .init(),
            ),
            input: self.input,
        }
    }
}

impl<Base, Workers> InsertOneOp<Base, Workers>
where
    Workers: InsertOneWorker,
{
    pub async fn exec_op(
        self,
        db: Pool<Sqlite>,
    ) -> Result<
        GetOneOutput<Base, TupleAsMap<Workers::Output>>,
        InsertError,
    > {
        todo!("use insert_one_dynamic instead")
    }
}

#[derive(Debug, Deserialize)]
pub struct InputInsertOne {
    pub input: Value,
    pub relation: Map<String, Value>,
}

#[derive(Debug, Serialize)]
pub struct OuputDynamic {
    id: i64,
    attr: Value,
    relations: Map<String, Value>,
}

#[axum::debug_handler]
pub async fn insert_one_dynamic(
    db: State<Pool<Sqlite>>,
    collection_name: Path<String>,
    input: Json<InputInsertOne>,
) -> Result<Json<OuputDynamic>, InsertError> {
    let collection_gaurd = COLLECTIONS.read().await;
    let relation_gaurd = RELATIONS.read().await;

    let collection = collection_gaurd
        .get(&collection_name.0.to_camel())
        .ok_or(error::entry_not_found(&collection_name.0))?;

    let (mut rels, tra) = {
        let mut rels = vec![];
        let mut trans = vec![];

        // key: snake_case
        'found: for (key, value) in input.0.relation.iter() {
            for r in relation_gaurd
                .get(collection.table_name())
                .unwrap()
            {
                match r
                    .clone()
                    .init_on_insert(&key, value.clone())
                {
                    DynamicRelationResult::Ok(ok) => {
                        rels.push(ok);
                        trans.push(key.clone());
                        continue 'found;
                    }
                    DynamicRelationResult::InvalidInput(err) => {
                        return Err(error::to_refactor(&format!(
                            "relation {key} invalid input for {}: {}",
                            collection.table_name(),
                            err
                        )))?
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

    let mut st = stmt::InsertStOne::init(
        collection.table_name().to_string(),
    );

    match collection.on_insert(input.0.input, &mut st) {
        Ok(()) => {}
        Err(ValidatedAndTyped::ValidationError(err)) => {
            return Err(error::to_refactor(&format!(
                "insert input failed validation: {}",
                err
            ))
            .into())
        }
        Err(ValidatedAndTyped::TypeError(err)) => {
            return Err(error::to_refactor(&format!(
                "insert input is invalid: {}",
                err
            ))
            .into())
        }
    }

    for rel in rels.iter_mut() {
        rel.sub_op1(db.0.clone()).await;
    }

    for rel in rels.iter_mut() {
        rel.on_insert(&mut st);
    }

    let mut res = st
        .returning(vec!["*"])
        .fetch_one(&db.0, |r| {
            let attr = collection.from_row_noscope(&r);
            let id: i64 = r.get("id");

            for rel in rels.iter_mut() {
                rel.from_row(&r);
            }

            Ok(OuputDynamic {
                id,
                attr,
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
