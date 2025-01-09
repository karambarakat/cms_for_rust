use std::collections::HashMap;

use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::Database;

use crate::entities::DynEntitySubmitable;

#[derive(Serialize, Deserialize)]
struct Field {
    name: String,
    type_: String,
}

#[derive(Serialize, Deserialize)]
struct Entity {
    name: String,
    fields: Vec<Field>,
}

// todo: fix the type of members
pub fn router<DB: Database>() -> Router {
    let router = Router::new().route(
        "/",
        get(|| async {
            let mut hash = HashMap::new();

            for each in
                inventory::iter::<DynEntitySubmitable<DB>>
            {
                let obj = (each.object)();
                let table_name = obj.table_name();
                let memebers = obj.members();
                hash.insert(
                    table_name,
                    Entity {
                        name: table_name.to_string(),
                        fields: memebers
                            .iter()
                            .map(|e| Field {
                                name: e.to_string(),
                                type_: "string".to_string(),
                            })
                            .collect(),
                    },
                );
            }

            Json(hash)
        }),
    );

    router
}
