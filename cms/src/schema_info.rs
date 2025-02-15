#![allow(unused)]
use axum::{routing::post, Json, Router};
use serde::Serialize;

use crate::dynamic_schema::SubmitDynCollection;

#[derive(Serialize)]
pub enum SupportedType {
    String,
    Todo,
}

#[derive(Serialize)]
pub struct Field {
    pub name: String,
    pub f_type: SupportedType,
}

#[derive(Serialize)]

pub struct Collection {
    pub name: String,
    pub fields: Vec<Field>,
}

#[axum::debug_handler]
async fn schema_handler_() -> Json<Vec<Collection>> {
    let mut v = vec![];

    for col in inventory::iter::<SubmitDynCollection> {
        let obj = (col.obj)();
        v.push(Collection {
            name: obj.table_name().to_string(),
            fields: obj
                .members_no_scope()
                .into_iter()
                .map(|e| Field {
                    name: e.to_string(),
                    f_type: SupportedType::Todo,
                })
                .collect(),
        });
    }

    Json(v)
}

pub fn schema_router() -> Router<()> {
    let app = Router::new().route("/", post(schema_handler_));

    app
}
