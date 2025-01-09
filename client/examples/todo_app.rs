use cms_for_rust::{
    entities::define as entity, relations::define as relations,
    schema,
};

schema! {
    db = "sqlx::Sqlite",
}

#[entity]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[entity]
pub struct Category {
    pub title: String,
}

#[entity]
pub struct Tag {
    pub title: String,
}

relations! {
    optional_to_many Todo Category;
    many_to_many Todo Tag;
}

fn main() {}

pub mod testing_prelude {
    #![allow(unused_imports)]
    #![allow(async_fn_in_trait)]

    use axum::response::Response;
    pub use axum::{
        body::Body,
        http::{Method, Request},
    };
    use futures::Future;
    pub use http_body_util::BodyExt;
    use sqlx::{Pool, Sqlite};
    pub use std::ops::Deref;
    use std::{error::Error, mem::swap};
    pub use tower::ServiceExt;
    use tracing::{debug, error};

    pub use axum::http::StatusCode;

    pub trait ToJson {
        fn into_json(
            self,
        ) -> impl futures::Future<
            Output = Result<
                serde_json::Value,
                serde_json::Error,
            >,
        >;
    }

    pub fn status_is_not(
        res: &mut Response<Body>,
        to_be: StatusCode,
    ) -> impl Future<Output = Result<(), String>> + '_ {
        async move {
            let expect = res.status();

            if expect == to_be {
                return Ok(());
            }

            // will panic, take the data from the body
            let mut new_body = Response::new(Body::empty());
            swap(res, &mut new_body);
            let res = new_body;

            let body_type = res
                .headers()
                .get("content-type")
                .map(|v| v.to_str().unwrap().to_string());

            let body_type = body_type.as_deref();

            let if_body =
                match res.into_body().collect().await.map(|v| {
                    std::str::from_utf8(&v.to_bytes())
                        .map(|e| e.to_string())
                }) {
                    Ok(Ok(str)) => Some(str),
                    // non-collectable or non-utf8
                    _ => None,
                };

            match (body_type, if_body) {
                (Some("application/json"), Some(body)) => {
                    Err(format!(
                        "failed at status code {}, body: {:?}",
                        expect, body
                    ))
                }
                (head, body) => Err(format!(
                    "failed at status code {}{}{}",
                    expect,
                    head.map(|v| format!(", head: {}", v))
                        .unwrap_or_default(),
                    body.map(|v| format!(", body: {}", v))
                        .unwrap_or_default(),
                )),
            }
        }
    }

    impl ToJson for axum::http::Response<Body> {
        fn into_json(
            self,
        ) -> impl futures::Future<
            Output = Result<
                serde_json::Value,
                serde_json::Error,
            >,
        > {
            async {
                let status = self.status();
                let body = match self
                    .into_body()
                    .collect()
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn Error>)
                {
                    Ok(body) => {
                        std::str::from_utf8(&body.to_bytes())
                            .map_err(|e| {
                                Box::new(e) as Box<dyn Error>
                            })
                            .map(|e| e.to_string())
                    }
                    Err(e) => Err(e),
                };

                match &body {
                    Ok(ok) => {
                        debug!("200 Ok {}", ok);
                    }
                    Err(err) => {
                        if status != StatusCode::OK {
                            error!("error {}: {}", status, err);
                        } else {
                            error!("error {}", status);
                        }
                    }
                }

                serde_json::from_str(
                    &body.expect(
                        "body is not convertable to string",
                    ),
                )
            }
        }
    }

    pub trait JsonBody {
        fn json_body(
            self,
            value: serde_json::Value,
        ) -> http::Result<Request<String>>;
    }

    impl JsonBody for http::request::Builder {
        fn json_body(
            self,
            value: serde_json::Value,
        ) -> http::Result<Request<String>> {
            self.header(
                http::header::CONTENT_TYPE,
                "application/json",
            )
            .body(
                serde_json::to_string(&value)
                    .expect("Failed to serialize json"),
            )
        }
    }

    pub async fn dumpy_data(pool: Pool<Sqlite>) {
        sqlx::query(
            "INSERT INTO Category (title) VALUES
    ('cat_1'), ('cat_2'), ('cat_3')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO Tag (title) VALUES
    ('tag_1'), ('tag_2'), ('tag_3'), ('tag_4')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
        "INSERT INTO Todo (title, done, description, category_id) VALUES
        ('todo_1', true, 'red', NULL),
        ('todo_2', false, 'green', 1),
        ('todo_3', false, 'blue', 3),
        ('todo_4', true, 'yellow', Null)",
    ).execute(&pool).await.unwrap();

        sqlx::query(
            "INSERT INTO TodosTags (todo_id, tag_id) VALUES
            (1, 1), (1, 2), (1, 3),
                    (2, 2), (2, 3),
            (4, 1),
            (3, 1), (3, 2)        ",
        )
        .execute(&pool)
        .await
        .unwrap();
    }
}

#[cfg(test)]
mod test {
    use std::{mem::swap, ops::Not};

    use crate::testing_prelude::*;
    use axum::{response::Response, routing::get, Router};
    use better_testing::{axum::invoking, expect, ToBe};
    use cms_for_rust::{
        axum_router::AxumRouter, migration::migrate,
    };
    use queries_for_sqlx::prelude::stmt;
    use serde_json::json;
    use sqlx::{Database, FromRow, Pool, Sqlite};

    use crate::*;

    #[tokio::test]
    async fn test_multi() {
        let pool =
            sqlx::Pool::<Sqlite>::connect("sqlite::memory:")
                .await
                .unwrap();

        cms_for_rust::migration::migrate(pool.clone()).await;
        dumpy_data(pool.clone()).await;

        // todo: remove this
        let app = axum::Router::new()
            .route("/", get(|| async { "Server is running" }))
            .nest("/todo", Todo::router())
            .nest("/category", Category::router())
            .with_state(pool.clone());

        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .init();

        test_get_all(app.clone()).await;
        test_get_one(app.clone()).await;
        test_insert_one(app.clone()).await;
        test_update_one(app.clone()).await;
        test_delete_one(app.clone()).await;
    }

    async fn test_get_all(app: Router) {
        let mut res = app
            .oneshot(
                Request::builder()
                    .uri("/todo")
                    .method(Method::GET)
                    .json_body(json!({
                        "query": {
                            "pagination": {
                                "page_size": 2,
                                "page_shift": 2,
                            },
                        },
                        "relations": {
                            "category": { "id": true, "attributes": true },
                            "tags": { "id": true, "attributes": true },
                        },
                    }))
                    .expect("request"),
            )
            .await
            .expect("oneshot");

        status_is_not(&mut res, StatusCode::OK).await.unwrap();
        let res = res.into_json().await.unwrap();

        expect(&res).to_be(&json!({
            "meta": { "page_number": 2 },
            "data": [
                {
                    "id": 3,
                    "attributes": {
                        "title": "todo_3",
                        "done": false,
                        "description": "blue",
                    },
                    "relations": {
                        "category": {
                            "id": 3,
                            "attributes": { "title": "cat_3" }
                        },
                        "tags": [
                            { "id": 1, "attributes": { "title": "tag_1" } },
                            { "id": 2, "attributes": { "title": "tag_2" } },
                        ]
                    }

                },
                {
                    "id": 4,
                    "attributes": {
                        "title": "todo_4",
                        "done": true,
                        "description": "yellow",
                    },
                    "relations": {
                        "category": null,
                        "tags": [
                            { "id": 1, "attributes": { "title": "tag_1" } },
                        ]
                    },
                },
            ]
        }));
    }

    async fn test_get_one(app: Router) {
        let mut res = app
            .oneshot(
                Request::builder()
                    .uri("/todo/one")
                    .method(Method::GET)
                    .json_body(json!({
                        "query": {
                            "id": 3
                        },
                        "relations": {
                            "category": { "id": true, "attributes": true },
                            "tags": { "id": true, "attributes": true },
                        },
                    }))
                    .expect("request"),
            )
            .await
            .expect("oneshot");

        status_is_not(&mut res, StatusCode::OK).await.unwrap();
        let res = res.into_json().await.unwrap();

        expect(&res).to_be(&json!(
            {
                "id": 3,
                "attributes": {
                    "title": "todo_3",
                    "done": false,
                    "description": "blue",
                },
                "relations": {
                    "category": {
                        "id": 3,
                        "attributes": { "title": "cat_3" }
                    },
                    "tags": [
                        { "id": 1, "attributes": { "title": "tag_1" } },
                        { "id": 2, "attributes": { "title": "tag_2" } },
                    ]
                }

            }
        ));
    }

    async fn test_insert_one(app: Router) {
        let mut res = app
            .oneshot(
                Request::builder()
                    .uri("/todo")
                    .method(Method::POST)
                    .json_body(json!({
                        "data": {
                            "title": "todo_5",
                            "done": true,
                            "description": "black",
                        },
                        "attributes": true,
                        "relations": {
                            "category": {
                                "data": { "id": 2 },
                                "id": true,
                                "attributes": true
                            },
                            "tags": {
                                 "data": [{ "id": 1 }, { "id": 2 }],
                                 "id": true,
                                 "attributes": true
                            },
                        },
                    }))
                    .expect("request"),
            )
            .await
            .expect("oneshot");

        status_is_not(&mut res, StatusCode::CREATED)
            .await
            .unwrap();
        let res = res.into_json().await.unwrap();

        expect(&res).to_be(&json!(
            {
                "id": 5,
                "attributes": {
                    "title": "todo_5",
                    "done": true,
                    "description": "black",
                },
                "relations": {
                    "category": {
                        "id": 2,
                        "attributes": { "title": "cat_2" }
                    },
                    "tags": [
                        { "id": 1, "attributes": { "title": "tag_1" } },
                        { "id": 2, "attributes": { "title": "tag_2" } },
                    ]
                }
            }
        ));
    }

    async fn test_delete_one(app: Router) {
        let mut res = app
            .oneshot(
                Request::builder()
                    .uri("/todo/one")
                    .method(Method::DELETE)
                    .json_body(json!({
                        "id": 2,
                        "attributes": true,
                        "relations_keys": true,
                    }))
                    .expect("request"),
            )
            .await
            .expect("oneshot");

        status_is_not(&mut res, StatusCode::OK).await.unwrap();
        let res = res.into_json().await.unwrap();

        expect(&res).to_be(&json!(
            {
                "id": 2,
                "attributes": {
                    "title": "todo_2",
                    "done": false,
                    "description": "green",
                },
                "relations_keys": {
                    "category_id": 1,
                }
            }
        ));
    }

    async fn test_update_one(app: Router) {
        let mut res = app
            .oneshot(
                Request::builder()
                    .uri("/todo/one")
                    .method(Method::PUT)
                    .json_body(json!({
                        "id": 3,
                        "data": {
                            "title": "new_title",
                        },
                        "attributes": true,
                        "relations": {
                            "category": {
                                "data": { "set": { "id": 2 } },
                                "id": true,
                                "attributes": true
                            },
                            "tags": {
                                "data": { "connect": { "id": [3] }, },
                                "attributes": true,
                                "id": true,
                            },
                        },
                    }))
                    .expect("request"),
            )
            .await
            .expect("oneshot");

        status_is_not(&mut res, StatusCode::OK).await.unwrap();
        let res = res.into_json().await.unwrap();

        expect(&res).to_be(&json!(
            {
                "id": 3,
                "attributes": {
                    "title": "new_title",
                    "done": false,
                    "description": "blue",
                },
                "relations": {
                    "category": {
                        "id": 2,
                        "attributes": { "title": "cat_2" }
                    },
                    "tags": [
                        { "id": 1, "attributes": { "title": "tag_1" } },
                        { "id": 2, "attributes": { "title": "tag_2" } },
                        { "id": 3, "attributes": { "title": "tag_3" } },
                    ]
                }
            }
        ));
    }
}
