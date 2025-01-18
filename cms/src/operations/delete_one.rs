use std::pin::Pin;

use axum::{
    extract::{Path, State},
    Json,
};
use case::CaseExt;
use queries_for_sqlx::prelude::*;
use serde_json::{Map, Value};
use sqlx::{sqlite::SqliteRow, Row};
use sqlx::{Pool, Sqlite};

use crate::{
    dynamic_schema::{
        DynamicRelationResult, COLLECTIONS, RELATIONS,
    },
    error::{self, GlobalError},
    queries_bridge::DeleteSt,
};

pub trait DynDeleteWorker: Send + Sync {
    fn sub_op(
        &mut self,
        db: State<Pool<Sqlite>>,
    ) -> Pin<Box<dyn std::future::Future<Output = ()> + Send>>;
    fn from_row(&mut self, r: &SqliteRow) -> Value;
}

#[derive(serde::Deserialize)]
pub struct DeleteInput {
    pub id: i64,
    pub return_attr: bool,
    pub return_residual: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct DeleteOutput {
    pub id: i64,
    pub attr: Value,
    pub relations: Map<String, Value>,
}

#[axum::debug_handler]
pub async fn delete_one_dynmaic(
    db: State<Pool<Sqlite>>,
    collection_name: Path<String>,
    input: Json<DeleteInput>,
) -> Result<Json<Option<DeleteOutput>>, GlobalError> {
    let collection_gaurd = COLLECTIONS.read().await;

    let collection = collection_gaurd
        .get(&collection_name.0.to_camel())
        .ok_or(error::entry_not_found(&collection_name.0))?;

    let mut st =
        DeleteSt::init(collection.table_name().to_string());

    st.where_(col("id").eq(input.id));

    let retrun_any =
        input.return_attr || !input.return_residual.is_empty();

    let ret = if input.return_attr {
        st.returning(vec!["*"])
    } else {
        st.returning(vec![])
    };

    let mut rels = if input.return_residual.is_empty() {
        None
    } else {
        let relation_gaurd = RELATIONS.read().await;
        let mut rels = Vec::new();
        let mut tra = Vec::new();

        'found: for key in input.0.return_residual.iter() {
            for r in relation_gaurd
                .get(collection.table_name())
                .unwrap()
            {
                match r
                    .clone()
                    .init_on_delete(&key)
                {
                    DynamicRelationResult::Ok(ok) => {
                        rels.push(ok);
                        tra.push(key.to_owned());
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

        for each in rels.iter_mut() {
            each.sub_op(db.clone()).await;
        }

        Some((relation_gaurd, rels, tra))
    };

    let res = ret
        .fetch_one(&db.0, |r| {
            if retrun_any {
                let mut relations = Map::default();
                let id = r.get("id");
                let attr = collection.from_row_noscope(&r);

                if let Some((_, rels, tra)) = &mut rels {
                    for rel in rels.into_iter().zip(tra.iter()) {
                        let res = rel.0.from_row(&r);
                        relations.insert(rel.1.to_owned(), res);
                    }
                }
                return Ok(Some(DeleteOutput {
                    id,
                    attr,
                    relations,
                }));
            }
            return Ok(None);
        })
        .await
        .unwrap();

    Ok(Json(res))
}
