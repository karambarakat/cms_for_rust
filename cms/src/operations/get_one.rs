use std::{collections::HashSet, ops::Not};

use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use case::CaseExt;
use inventory::Collect;
use queries_for_sqlx::{
    execute_no_cache::ExecuteNoCache,
    impl_into_mut_arguments_prelude::Type,
    prelude::{
        col, ft, stmt, verbatim__warning__does_not_sanitize,
        SelectHelpers,
    },
    quick_query::QuickQuery,
    select_st::SelectSt,
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
    relations::{get_relation, other::OneKey},
    entities::Entity,
};

use super::Id;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectInput<E, Q, R> {
    #[serde(skip)]
    pub entity: E,
    pub query: Q,
    pub relations: R,
}

#[derive(Debug, Deserialize)]
pub struct SupportedQueriesOnGetOne {
    pub id: i64,
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

pub async fn get_one<S, T>(
    pool: State<Pool<S>>,
    Json(input): Json<
        SelectInput<
            (),
            SupportedQueriesOnGetOne,
            Map<String, Value>,
        >,
    >,
) -> Result<Response<Body>, MyError>
where
    S: Send + 'static,
    S: Database,
    T: Entity<S> + Serialize + Send,
    S: SupportNamedBind,
    for<'c> &'c mut S::Connection: Executor<'c, Database = S>,
    for<'r> &'r str: ColumnIndex<S::Row>,
    crate::relations::Submitable<S>: Collect,
    i64: Type<S> + for<'d> Decode<'d, S> + for<'d> Encode<'d, S>,
{
    let mut rels = match get_relation::<S>(
        T::table_name().to_snake().as_str(),
        "from_get_one",
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

    let mut st = stmt::select(T::table_name());

    st.where_(col("local_id").eq(move || input.query.id));

    st.select(
        ft(T::table_name()).col("id").prefix_alias("local"),
    );

    for rel in rels.iter_mut() {
        rel.on_select(&mut st);
    }

    for (alias, member) in
        T::members_scoped().into_iter().zip(T::members())
    {
        st.select(ft(T::table_name()).col(member).alias(alias));
    }

    let mut data = st
        .fetch_optional(&pool.0.clone(), |row| {
            let attributes = T::from_row_scoped(&row)?;

            let mut relation = serde_json::Map::default();

            for rel in rels.iter_mut() {
                rel.from_row(&row);
                if let Some(value) = rel.take() {
                    relation
                        .entry(rel.schema_key())
                        .or_insert(value);
                }
            }

            let local_id: i64 = row.get("local_id");

            Ok((local_id, attributes, relation))
            // Ok(json!({
            //     "id": local_id,
            //     "attributes": attributes,
            //     "relations": relation,
            // }))
        })
        .await
        .unwrap();

    let mut data = if let Some(data) = data {
        data
    } else {
        return Err(error(StatusCode::NOT_FOUND).for_dev(
            format!(
                "no {} with id {}",
                T::table_name(),
                input.query.id
            ),
        ));
    };

    for rel in rels.iter_mut() {
        rel.sub_op(
            pool.0.clone(),
            std::sync::Arc::new(OneKey { id: data.0 }),
        )
        .await;
    }

    for rel in rels.iter_mut() {
        if let Some(ouput) = rel.take2(data.0) {
            data.2.entry(rel.schema_key()).or_insert(ouput);
        }
    }

    let output = json!({
        "id": data.0,
        "attributes": data.1,
        "relations": data.2,
    });

    Ok(Json(output).into_response())
}
