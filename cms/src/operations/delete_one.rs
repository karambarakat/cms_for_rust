use std::{collections::HashSet, ops::Not};

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
    IntoMutArguments, SupportNamedBind, SupportReturning,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sqlx::{
    ColumnIndex, Database, Decode, Encode, Executor, Pool, Row,
};
use tracing::warn;

use crate::{
    axum_router::{error, MyError},
    relations::get_relation,
};

use super::Id;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteInput<E, Q, Returning, RK> {
    #[serde(skip)]
    pub entity: E,
    #[serde(flatten)] // only have id in delete_one
    pub query: Q,
    #[serde(rename = "attributes")]
    pub return_data: Returning,
    pub relations_keys: RK,
}

pub async fn delete_one<S, T>(
    pool: State<Pool<S>>,
    Json(input): Json<
        DeleteInput<
            // entity: dynaimc dispatch
            (),
            // query: now only supports id
            Id,
            // return data:
            bool,
            bool,
        >,
    >,
) -> Result<Response<Body>, MyError>
where
    S: Database + SupportReturning + SupportNamedBind,
    T: crate::entities::Entity<S> + Serialize,
    for<'c> &'c mut <S as Database>::Connection:
        Executor<'c, Database = S>,
    for<'r> &'r str: ColumnIndex<S::Row>,
    i64: Type<S> + for<'d> Decode<'d, S> + for<'d> Encode<'d, S>,
{
    let mut st = stmt::delete(T::table_name());

    let id = input.query.id;
    st.where_(col("id").eq(move || id));

    if input.return_data {
        let mut output = json!({
            "id": id,
            "data": null,
            "relations_keys": {},
        });
        let mut relations_keys = serde_json::Map::default();
        let data = st
            .returning(vec!["*"])
            .fetch_optional(&pool.0, |row| {
                use sqlx::Column;
                use sqlx::Row;

                let mut cols = row
                    .columns()
                    .iter()
                    .map(|e| e.name())
                    .collect::<HashSet<_>>();

                cols.remove("id");
                let member = HashSet::from_iter(T::members());
                let cols = cols.difference(&member);

                for member in cols {
                    let id: Option<i64> = row.get(member);
                    relations_keys
                        .insert(member.to_string(), id.into());
                }

                let t = T::from_row(&row)?;

                return Ok(t);
            })
            .await
            .unwrap();

        match data {
            Some(data) => Ok(Json(json!({
                "id": id,
                "attributes": data,
                "relations_keys": relations_keys,
            }))
            .into_response()),
            None => Ok(Json(json!({})).into_response()),
        }
    } else {
        let id = st
            .returning(vec!["id"])
            .fetch_optional(&pool.0, |r| {
                let id: i64 = r.get("id");
                Ok(id)
            })
            .await
            .unwrap();

        Ok(Json(json!({ "id": id })).into_response())
    }
}
