use axum::{
    body::Body,
    extract::{Request, State},
    http::{Response, StatusCode},
    middleware::from_fn,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use serde_json::json;
use sqlx::{Pool, Sqlite};
use std::{
    collections::HashMap, convert::Infallible, future::Future,
    marker::PhantomData, ops::Not, sync::RwLock,
};
use tower::Service;

use crate::auth::need_super_user;

pub fn admin_router() -> Router<Pool<Sqlite>> {
    let app = Router::new();

    let mut app = app
        .route("/inspect_schema", get(inspect_schema))
        .route_layer(from_fn(need_super_user));

    app
}

#[derive(Serialize)]
struct Schema {}

#[axum::debug_handler]
async fn inspect_schema() -> Json<Schema> {
    Json(Schema {})
}
