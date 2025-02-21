use axum::{
    body::Body,
    extract::Request,
    http::{header::CONTENT_TYPE, Response},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use http_body_util::BodyExt;
use serde_json::json;

#[axum::debug_middleware]
pub async fn uniform_response_middleware(
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let res = next.run(req).await;
    let s = res.status();
    if s.is_client_error() || s.is_server_error() {
        if let Some(ct) = res.headers().get(CONTENT_TYPE) {
            if ct == "text/plain; charset=utf-8" {
                let (mut parts, body) = res.into_parts();
                let body = String::from_utf8(
                    body.collect()
                        .await
                        .unwrap()
                        .to_bytes()
                        .into(),
                )
                .unwrap();
                parts.headers.insert(
                    CONTENT_TYPE,
                    "application/json".parse().unwrap(),
                );
                let body = format!("Unknown error: {}", body);
                return (
                    parts,
                    Json(json!({
                        "error": {
                            "hint": body,
                            "user_error": null,
                        },
                    })),
                )
                    .into_response();
            }
        }
    }

    res
}
