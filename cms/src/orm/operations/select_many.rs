use std::future::Future;

use axum::{
    extract::{Path, State},
    Json,
};
use case::CaseExt;
use queries_for_sqlx::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::{sqlite::SqliteRow, Row};
use sqlx::{Pool, Sqlite};

use crate::orm::{
    dynamic_schema::{
        DynamicRelationResult, COLLECTIONS, RELATIONS,
    }, error::{self, GlobalError}, queries::{AgnosticFilter, Filters}, queries_bridge::SelectSt
};

use super::select_one::{GetOneOuputDynamic, GetOneWorker};

pub trait GetAllWorker: Sync + Send {
    type Inner: Default + Send + Sync;
    type Output;
    fn on_select(
        &self,
        data: &mut Self::Inner,
        st: &mut SelectSt,
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
    fn take(
        &mut self,
        current_id: i64,
        data: &mut Self::Inner,
    ) -> Self::Output;
}

#[derive(Debug, Deserialize)]
pub struct InputGetMany {
    pub filters: Map<String, Value>,
    pub relations: Map<String, Value>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination {
    pub page: i32,
    pub page_size: i32,
}

#[derive(Serialize)]
pub struct GetManyOutputDynamic {
    data: Vec<GetOneOuputDynamic>,
    page_count: (),
}

#[axum::debug_handler]
pub async fn get_all_dynamic(
    db: State<Pool<Sqlite>>,
    collection_name: Path<String>,
    input: Json<InputGetMany>,
) -> Result<Json<GetManyOutputDynamic>, GlobalError> {
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
                    .init_on_get_all(&key, value.clone())
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
        SelectSt::init(collection.table_name().to_string());

    for rel in rels.iter_mut() {
        rel.on_select(&mut st);
    }

    collection.on_select(&mut st);

    input.0.pagination.on_select(&mut st);

    st.select(
        ft(collection.table_name().to_string())
            .col("id".to_string())
            .alias("local_id".to_string()),
    );

    let mut res = st
        .fetch_all(&db.0, |r| {
            let attr = collection.from_row_scoped(&r);

            let id: i64 = r.get("local_id");

            for rel in rels.iter_mut() {
                rel.from_row(&r)
            }

            return Ok(GetOneOuputDynamic {
                id,
                attr,
                relations: Default::default(),
            });
        })
        .await
        .unwrap();

    for mut rel in rels.iter_mut() {
        rel.sub_op(db.0.clone()).await;
    }

    for (mut rel, t) in rels.into_iter().zip(tra) {
        for each in res.iter_mut() {
            let taken = rel.take(each.id);
            each.relations.insert(t.to_string(), taken);
        }
    }

    // hold the gaurds, to not change the schema while holding old schema objects
    drop(collection_gaurd);
    drop(relation_gaurd);

    Ok(Json(GetManyOutputDynamic {
        data: res,
        page_count: (),
    }))
}
