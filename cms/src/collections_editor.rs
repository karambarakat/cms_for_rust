#![allow(unused)]
use std::{
    collections::HashMap, convert::Infallible, future::Future,
    marker::PhantomData, ops::Not, sync::RwLock,
};

use auth::authenticated;
use axum::{
    body::Body,
    extract::{Request, State},
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde_json::json;
use sqlx::{Pool, Sqlite};
use tower::Service;

use crate::testing::TestHandler;

mod auth {
    use std::convert::Infallible;

    use axum::{
        extract::FromRequestParts,
        http::request::Parts,
        middleware::{from_extractor, FromExtractorLayer},
    };

    pub(super) struct Auth;

    impl FromRequestParts<()> for Auth {
        type Rejection = Infallible;

        async fn from_request_parts(
            parts: &mut Parts,
            state: &(),
        ) -> Result<Self, Self::Rejection> {
            todo!()
        }
    }

    pub(super) fn authenticated() -> FromExtractorLayer<Auth, ()>
    {
        from_extractor::<Auth>()
    }
}

pub async fn todo_handler() -> StatusCode {
    StatusCode::NOT_FOUND
}

lazy_static::lazy_static! {
    static ref NEED_SUPERUSER: tokio::sync::RwLock<bool> = tokio::sync::RwLock::new(true);
}

async fn zero_superusers(
    db: State<Pool<Sqlite>>,
) -> Response<Body> {
    let need_s = NEED_SUPERUSER.read().await;
    if *need_s {
        return Json(json!({"zero_superusers": true}))
            .into_response();
    }

    drop(need_s);
    let mut mod_need_s_guard = NEED_SUPERUSER.write().await;

    let query: (i32,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) as count FROM superusers
        "#,
    )
    .fetch_one(&db.0)
    .await
    .unwrap();

    if query.0 == 0 {
        return Json(json!({"zero_superusers": true}))
            .into_response();
    } else {
        *mod_need_s_guard = false;
        return Json(json!({"zero_superusers": false}))
            .into_response();
    }
}

// #[tokio::test]
// async fn test_zero_superusers() {
//     let pool = Pool::<Sqlite>::connect("sqlite::memory:")
//         .await
//         .unwrap();
//
//     let test_router =
//         TestHandler::new(zero_superusers, move |router| {
//             router.layer(authenticated()).with_state(pool)
//         });
//
//     let response = test_router.json_test(json!(null)).await;
//
//     assert_eq!(response.body(), r#"{"zero_superusers":true}"#);
// }

pub fn router(pool: Pool<Sqlite>) -> Router<Pool<Sqlite>> {
    let app = Router::new();

    let mut app = app
        .route("/need_superuser", post(zero_superusers))
        .route("/add_entity", post(todo_handler))
        .route("/delete_entity", post(todo_handler))
        .route("/rename_entity", post(todo_handler)) // mod_entity
        .route("/add_field", post(todo_handler)) // mod_entity
        .route("/delete_field", post(todo_handler)) // mod_entity
        .route("/mod_field", post(todo_handler)) // mod_entity
        .route("/add_relation", post(todo_handler))
        .route("/delete_relation", post(todo_handler))
        .route_layer(authenticated());

    app
}
