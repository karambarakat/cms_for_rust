use std::ops::Not;

use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use case::CaseExt;
use inventory::Collect;
use queries_for_sqlx::{execute_no_cache::ExecuteNoCache, InitStatement};
use queries_for_sqlx::{
    impl_into_mut_arguments_prelude::Type, prelude::stmt,
    IntoMutArguments, SupportNamedBind, SupportReturning,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sqlx::Row;
use sqlx::{
    ColumnIndex, Database, Decode, Encode, Executor, Pool,
};
use tracing::warn;

use crate::{
    axum_router::{error, MyError},
    entities::Entity,
    relations::get_relation,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InsertInput<E, D, R> {
    #[serde(skip)]
    pub entity: E,
    pub data: D,
    pub attributes: bool,
    pub relations: R,
}

pub async fn insert_one<S, T>(
    pool: State<Pool<S>>,
    Json(input): Json<InsertInput<(), T, Map<String, Value>>>,
) -> Result<Response<Body>, MyError>
where
    S: Database,
    T: Entity<S>
        + Serialize
        + Send
        + for<'d> Deserialize<'d>
        + Clone
        + for<'q> IntoMutArguments<'q, S>,
    S: SupportNamedBind + SupportReturning,
    for<'c> &'c mut S::Connection: Executor<'c, Database = S>,
    for<'r> &'r str: ColumnIndex<S::Row>,
    i64: Type<S> + for<'d> Decode<'d, S> + for<'d> Encode<'d, S>,
    crate::relations::Submitable<S>: Collect,
{
    if input.attributes.not() {
        panic!(
            "not returning attributes is not supported for now"
        );
    }

    let mut rels = match get_relation::<S>(
        T::table_name().to_snake().as_str(),
        "from_insert_one",
        input.relations,
    ) {
        Ok(e) => e,
        Err(e) => {
            return Err(error(StatusCode::BAD_REQUEST).for_dev(
                format!(
                "following relations ({}) are not related to {}",
                e.join(", "),
                T::table_name()
            ),
            ));
        }
    };

    let mut st = stmt::InsertStOne::init(T::table_name());

    input.data.clone().insert_st(&mut st);

    for rel in rels.iter_mut() {
        rel.on_insert(&mut st);
    }

    // I will rely on cloning for now the input to output instead of populating for now
    let res = st
        .returning(vec!["id"])
        .fetch_one(&pool.0, |r| {
            let id: i64 = r.get("id");
            warn!("refactor: from_row and take should not exist in insert_one");
            Ok(id)
        })
        .await
        .unwrap();

    for rel in rels.iter_mut() {
        rel.sub_op2(pool.0.clone(), res).await;
    }

    let mut relation = serde_json::Map::default();
    for rel in rels.iter_mut() {
        warn!("refactor: take2 should not take any id on insert_one");
        if let Some(value) = rel.take2(Default::default()) {
            relation.entry(rel.schema_key()).or_insert(value);
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": res,
            "attributes": input.data,
            "relations": relation,
        })),
    )
        .into_response())
}

#[cfg(test)]
mod test_insert_one {
    fn main() {
        panic!("test not implemented");
    }
}
