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
    relations::{get_relation, other::BaseKeySet},
    entities::Entity,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectAllInput<E, Q, R> {
    #[serde(skip)]
    pub entity: E,
    pub query: Q,
    pub relations: R,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pagination {
    pub page_size: i64,
    pub page_shift: i64,
}

#[derive(Debug, Deserialize)]
// #[serde(deny_unknown_fields)]
pub struct QueriesSupportedByGetAll {
    pub pagination: Option<Pagination>,
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

pub async fn get_all<S, T>(
    pool: State<Pool<S>>,
    Json(input): Json<
        SelectAllInput<
            (),
            QueriesSupportedByGetAll,
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
    i64: Type<S> + for<'d> Decode<'d, S> + for<'d> Encode<'d, S>,
{
    let mut rels = match get_relation::<S>(
        T::table_name().to_snake().as_str(),
        "from_get_all",
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

    st.select(
        ft(T::table_name()).col("id").prefix_alias("local"),
    );

    for rel in rels.iter_mut() {
        rel.on_select(&mut st);
    }

    let page_number =
        if let Some(pagination) = input.query.pagination {
            st.offset(move || pagination.page_shift);
            st.limit(move || pagination.page_size);
            pagination.page_shift
        } else {
            0
        };

    for (alias, member) in
        T::members_scoped().into_iter().zip(T::members())
    {
        st.select(ft(T::table_name()).col(member).alias(alias));
    }

    let mut data = st
        .fetch_all(&pool.0.clone(), |row| {
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

    let where_ = data
        .iter()
        .map(|e| e.0.to_string())
        .collect::<Vec<_>>()
        .join(" OR ");

    for rel in rels.iter_mut() {
        rel.sub_op(
            pool.0.clone(),
            std::sync::Arc::new(BaseKeySet {
                vec: data.iter().map(|e| e.0).collect(),
            }),
        )
        .await;
    }

    let mut data2 = vec![];

    for mut val in data {
        // let mut rel_map = &mut val.2;
        let mut local_id = val.0;

        for rel in rels.iter_mut() {
            if let Some(ouput) = rel.take2(local_id) {
                val.2.entry(rel.schema_key()).or_insert(ouput);
            }
        }

        data2.push(json!({
            "id": local_id,
            "attributes": val.1,
            "relations": val.2,
        }))
    }

    drop(rels);

    let mut output = json!({
        "meta": { "page_number": page_number },
        "data": data2
    });

    // ziping
    Ok(Json(output).into_response())
}
