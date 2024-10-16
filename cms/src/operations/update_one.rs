use std::ops::Not;

use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use case::CaseExt;
use queries_for_sqlx::{
    execute_no_cache::ExecuteNoCache,
    impl_into_mut_arguments_prelude::Type,
    prelude::{col, stmt},
    InitStatement, SupportNamedBind, SupportReturning,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sqlx::{
    ColumnIndex, Database, Decode, Encode, Executor, Pool,
};
use tracing::warn;

use crate::{
    axum_router::{error, MyError},
    entities::Entity,
    entities::PartialEntity,
    relations::get_relation,
};

use super::Id;
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateInput<E, Q, D, R, Returning> {
    #[serde(skip)]
    pub entity: E,
    #[serde(flatten)] // only have id in update_one
    pub query: Q,
    pub data: D,
    pub relations: R,
    #[serde(rename = "attributes")]
    pub return_data: Returning,
}

pub async fn update_one<S, T>(
    pool: State<Pool<S>>,
    Json(input): Json<
        UpdateInput<
            // entity: dynaimc dispatch
            (),
            Id,
            // data:
            T::Partial,
            // relatinos:
            Map<String, Value>,
            // return data:
            bool,
        >,
    >,
) -> Result<Response<Body>, MyError>
where
    S: Database + SupportNamedBind + SupportReturning,
    T: Entity<S> + Serialize,
    T::Partial:
        PartialEntity<S> + for<'d> Deserialize<'d> + Send,
    for<'c> &'c mut <S as Database>::Connection:
        Executor<'c, Database = S>,
    for<'r> &'r str: ColumnIndex<S::Row>,
    i64: Type<S> + for<'d> Decode<'d, S> + for<'d> Encode<'d, S>,
{
    if input.return_data.not() {
        panic!(
            "not returning attributes is not supported for now"
        );
    }

    let mut rels = match get_relation::<S>(
        T::table_name().to_snake().as_str(),
        "from_update_one",
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

    let mut st = stmt::UpdateSt::init(T::table_name());

    let id = input.query.id;
    st.where_(col("id").eq(move || id));

    input.data.update_st(&mut st);

    for rel in rels.iter_mut() {
        rel.on_update(&mut st);
    }

    let res = st
        .returning(vec!["*"])
        .fetch_one(&pool.0, |row| {
            let value = T::from_row(&row)?;
            let value = serde_json::to_value(value).unwrap();
            warn!("refactor: from_row and take should not exist in update_one");
            Ok(value)
        })
        .await
        .unwrap();

    for rel in rels.iter_mut() {
        rel.sub_op2(pool.0.clone(), id).await;
    }

    let mut relation = serde_json::Map::default();
    for rel in rels.iter_mut() {
        warn!("refactor: take2 should not take any id on insert_one");
        if let Some(value) = rel.take2(Default::default()) {
            relation.entry(rel.schema_key()).or_insert(value);
        }
    }

    Ok(Json(json!({
        "id": id,
        "attributes": res,
        "relations": relation,
    }))
    .into_response())
}
