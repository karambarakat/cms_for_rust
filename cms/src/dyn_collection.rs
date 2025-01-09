#![allow(unused)]
use sqlx::{sqlite::SqliteRow, Row};
use std::{
    collections::HashMap,
    convert::Infallible,
    error::Error,
    future::Future,
    marker::PhantomData,
    ops::Not,
    sync::{Arc, RwLock},
};

use crate::{
    client_example::*,
    coll::dynamic::DynCollection,
    collections::error::{self, CollectionError},
};
use axum::{
    body::Body,
    extract::{self, Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serializer};
use serde_json::{json, Map, Value};
use sqlx::{Execute, Pool, Sqlite};

use queries_for_sqlx::{
    self, ident_safety::PanicOnUnsafe, prelude::*,
    quick_query::QuickQuery,
};
use tower::Service;

use crate::{
    coll::{
        dynamic::TrivialCollection,
        rel_optional_to_many::OptionalToMany,
        rel_trait::{DynRelation, Operation},
    },
    collections_editor::todo_handler,
};

#[derive(Debug, Deserialize)]
struct InputGetOne {
    pub query: Map<String, Value>,
    pub relation: Map<String, Value>,
    pub id: i64,
}
async fn todo() -> impl IntoResponse {
    StatusCode::NOT_FOUND
}

lazy_static::lazy_static! {
    static ref RELATIONS: tokio::sync::RwLock<HashMap<String, Vec<Arc<dyn DynRelation + Send + Sync>>>> = {
        let all:
        Vec<Arc<dyn DynRelation + Send + Sync>>
        = vec![
            // Arc::new(OneToMany::<Todo, Category>::new()),
            // Arc::new(OptionalToMany::<Todo, Tag>::new(todo!())),
        ];

        tokio::sync::RwLock::new(
            HashMap::from_iter(vec![
                ("todo".to_string(), vec![all[1].clone(),all[0].clone()]),
                ("tag".to_string(), vec![]),
            ])
        )
    };
}

async fn get_one(
    db: State<Pool<Sqlite>>,
    input: Json<InputGetOne>,
) -> Result<Response<Body>, CollectionError> {
    let id = input.0.id.try_into().unwrap();
    let collection = TrivialCollection::<Todo>::new();

    let mut st = stmt::SelectSt::init(
        collection.table_name_s().to_owned(),
    );

    collection.on_select(&mut st);

    // keep the guard alive as long as you hold Box<dyn On*Relation>
    let rel_guard = RELATIONS.read().await;
    let mut rels = {
        let mut rel_keys =
            input.0.relation.into_iter().collect::<Vec<_>>();
        let mut rel = vec![];
        for r in rel_guard.get("Todo").unwrap() {
            let mut found = None;
            for (key, value) in rel_keys.iter() {
                match r.clone().init_get(
                    Operation::SelectOne,
                    key,
                    value.clone(),
                ) {
                    Ok(iiii) => {
                        found = Some(iiii);
                        break;
                    }
                    Err(err) => {
                        return Err(error::to_refactor(
                            StatusCode::BAD_REQUEST,
                        )
                        .for_dev(err))
                    }
                }
            }
            if let Some(iiii) = found {
                rel.push(iiii);
            } else {
                return Err(error::to_refactor(
                    StatusCode::BAD_REQUEST,
                ));
            }
        }
        rel
    };

    for rel in rels.iter_mut() {
        rel.on_select(&mut st);
    }

    let mut data = st
        .fetch_optional(&db.0, |mut r| {
            let attr = collection
                .on_get(&mut r)
                .expect("todo handle non-sqlx errors");

            for rel in rels.iter_mut() {
                rel.from_row(&mut r);
            }

            Ok(attr)
        })
        .await
        .unwrap();

    let data = data.ok_or(error::not_found(id))?;

    for rel in rels.iter_mut() {
        rel.sub_op(db.0.clone()).await;
    }

    let mut rel_res = Map::default();

    for mut rel in rels {
        rel_res
            .insert(rel.key().to_string(), rel.take(id.into()));
    }

    drop(rel_guard);

    Ok(Json(json!({
        "id": id,
        "data": data,
        "relations": rel_res,
    }))
    .into_response())
}

#[derive(Debug, Deserialize)]
struct InputGetAll {
    pub query: Map<String, Value>,
    pub relation: Map<String, Value>,
    pub pagination: Pagination,
}

#[derive(Debug, Deserialize)]
struct Pagination {
    pub page_size: i64,
    pub page_shift: i64,
}

async fn get_all(
    db: State<Pool<Sqlite>>,
    input: Json<InputGetOne>,
) -> Result<Response<Body>, CollectionError> {
    let collection = TrivialCollection::<Todo>::new();

    let mut st = stmt::SelectSt::init(collection.table_name_s().to_owned());

    collection.on_select(&mut st);

    // keep the guard alive as long as you hold Box<dyn On*Relation>
    let rel_guard = RELATIONS.read().await;
    let mut rels = {
        let mut rel_keys =
            input.0.relation.into_iter().collect::<Vec<_>>();
        let mut rel = vec![];
        for r in rel_guard.get("Todo").unwrap() {
            let mut found = None;
            for (key, value) in rel_keys.iter() {
                match r.clone().init_get(
                    Operation::SelectMany,
                    key,
                    value.clone(),
                ) {
                    Ok(iiii) => {
                        found = Some(iiii);
                        break;
                    }
                    Err(err) => {
                        return Err(error::to_refactor(
                            StatusCode::BAD_REQUEST,
                        )
                        .for_dev(err))
                    }
                }
            }
            if let Some(iiii) = found {
                rel.push(iiii);
            } else {
                return Err(error::to_refactor(
                    StatusCode::BAD_REQUEST,
                ));
            }
        }
        rel
    };

    for rel in rels.iter_mut() {
        rel.on_select(&mut st);
    }

    let mut data = st
        .fetch_all(&db.0, |mut r| {
            let id: i64 = r.get("id");

            let attr = collection
                .on_get(&mut r)
                .expect("todo handle non-sqlx errors");

            for rel in rels.iter_mut() {
                rel.from_row(&mut r);
            }

            Ok((id, attr))
        })
        .await
        .unwrap();

    for rel in rels.iter_mut() {
        rel.sub_op(db.0.clone()).await;
    }

    let mut res = vec![];

    for (id, data) in data {
        let mut relations = Map::default();
        for mut rel in rels.iter_mut() {
            relations
                .insert(rel.key().to_string(), rel.take(id));
        }
        res.push(json!({
            "id": id,
            "data": data,
            "relations": relations,
        }));
    }

    drop(rel_guard);

    Ok(Json(res).into_response())
}

#[derive(Debug, Deserialize)]
struct InputInsertOne {
    pub data: Value,
    pub attributes: bool,
    pub relation: Map<String, Value>,
}

async fn insert_one(
    db: State<Pool<Sqlite>>,
    input: Json<InputInsertOne>,
) -> Result<Response<Body>, CollectionError> {
    if input.0.attributes.not() {
        panic!("attributes is required");
    }

    let collection = TrivialCollection::<Todo>::new();

    let mut st =
        stmt::InsertStOne::init(collection.table_name_s());

    collection.on_insert(&mut st, input.0.data);

    // keep the guard alive as long as you hold Box<dyn On*Relation>
    let rel_guard = RELATIONS.read().await;
    let mut rels = {
        let mut rel_keys =
            input.0.relation.into_iter().collect::<Vec<_>>();
        let mut rel = vec![];
        for r in rel_guard.get("Todo").unwrap() {
            let mut found = None;
            for (key, value) in rel_keys.iter() {
                match r.clone().init_insert(key, value.clone()) {
                    Ok(iiii) => {
                        found = Some(iiii);
                        break;
                    }
                    Err(err) => {
                        return Err(error::to_refactor(
                            StatusCode::BAD_REQUEST,
                        )
                        .for_dev(err))
                    }
                }
            }
            if let Some(iiii) = found {
                rel.push(iiii);
            } else {
                return Err(error::to_refactor(
                    StatusCode::BAD_REQUEST,
                ));
            }
        }
        rel
    };

    for rel in rels.iter_mut() {
        rel.on_insert(&mut st);
    }
    for rel in rels.iter_mut() {
        rel.sub_op_1(db.0.clone()).await;
    }

    let res = st
        .returning(collection.on_insert_returning())
        .fetch_one(&db.0, |mut r| {
            let data = collection
                .on_get(&mut r)
                .expect("todo handle non-sqlx errors");

            for rel in rels.iter_mut() {
                rel.from_row(&mut r);
            }

            Ok(data)
        })
        .await
        .unwrap();

    for rel in rels.iter_mut() {
        rel.sub_op_2(db.0.clone()).await;
    }

    let mut relations = Map::default();

    for rel in rels.iter_mut() {
        relations.insert(rel.key().to_string(), rel.take());
    }

    drop(rel_guard);

    let res = json!({
        "data": res,
        "relations": relations,
    });

    Ok((StatusCode::CREATED, Json(res)).into_response())
}

#[derive(Debug, Deserialize)]
struct InputUpdate {
    pub id: i64,
    pub data: Value,
    pub attributes: bool,
    pub relation: Map<String, Value>,
}

async fn update_one(
    db: State<Pool<Sqlite>>,
    input: Json<InputUpdate>,
) -> Result<Response<Body>, CollectionError> {
    if input.0.attributes.not() {
        panic!("attributes is required");
    }

    let collection = TrivialCollection::<Todo>::new();

    let mut st = stmt::UpdateSt::init(collection.table_name_s());

    collection.on_update(&mut st, input.0.data);

    // keep the guard alive as long as you hold Box<dyn On*Relation>
    let rel_guard = RELATIONS.read().await;
    let mut rels = {
        let mut rel_keys =
            input.0.relation.into_iter().collect::<Vec<_>>();
        let mut rel = vec![];
        for r in rel_guard.get("Todo").unwrap() {
            let mut found = None;
            for (key, value) in rel_keys.iter() {
                match r.clone().init_update(key, value.clone()) {
                    Ok(iiii) => {
                        found = Some(iiii);
                        break;
                    }
                    Err(err) => {
                        return Err(error::to_refactor(
                            StatusCode::BAD_REQUEST,
                        )
                        .for_dev(err))
                    }
                }
            }
            if let Some(iiii) = found {
                rel.push(iiii);
            } else {
                return Err(error::to_refactor(
                    StatusCode::BAD_REQUEST,
                ));
            }
        }
        rel
    };

    for rel in rels.iter_mut() {
        rel.on_update(&mut st);
    }

    let id = input.0.id;
    st.where_(col("id".to_string()).eq(move || input.0.id));

    let res = st
        .returning_(collection.on_insert_returning())
        .fetch_one(&db.0, |mut r| {
            let data = collection
                .on_get(&mut r)
                .expect("todo handle non-sqlx errors");

            for rel in rels.iter_mut() {
                rel.from_row(&r);
            }

            Ok(json!({
                "data": data,
            }))
        })
        .await
        .unwrap();

    for rel in rels.iter_mut() {
        rel.sub_op(db.0.clone()).await;
    }

    let mut relations = Map::default();

    for rel in rels.iter_mut() {
        relations.insert(rel.key().to_string(), rel.take());
    }

    drop(rel_guard);

    Ok(Json(json!({
        "id": id,
        "data": res,
        "relations": relations,
    }))
    .into_response())
}

#[derive(Debug, Deserialize)]
struct InputDelete {
    pub id: i64,
    pub return_data: bool,
    pub return_relation_keys: bool,
}

async fn delete_one(
    db: State<Pool<Sqlite>>,
    input: Json<InputDelete>,
) -> Result<Response<Body>, CollectionError> {
    let collection = TrivialCollection::<Todo>::new();

    let mut st = stmt::DeleteSt::<_, QuickQuery, _>::init(
        collection.table_name_s(),
    );

    let id = input.0.id;
    st.where_(col("id".to_string()).eq(move || input.0.id));

    if input.0.return_data || input.0.return_relation_keys {
        panic!("return_data and return_relation_keys are not implemented");
    }

    st.returning(vec!["id"])
        .fetch_optional(&db.0, |r| {
            let id: i64 = r.get("id");
            Ok(id)
        })
        .await
        .unwrap();

    Ok(Json(json!({
        "id": id,
    }))
    .into_response())
}

pub fn router() -> Router<Pool<Sqlite>> {
    let app = Router::new()
        .route("/:collection/get_all", post(get_all))
        .route("/:collection/get_one", post(get_one))
        .route("/:collection/insert_one", post(insert_one))
        .route("/:collection/update_one", post(update_one))
        .route("/:collection/delete_one", post(delete_one));

    app
}
