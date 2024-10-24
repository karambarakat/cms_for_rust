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
        Output = Result<serde_json::Value, serde_json::Error>,
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
        Output = Result<serde_json::Value, serde_json::Error>,
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
                &body
                    .expect("body is not convertable to string"),
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
