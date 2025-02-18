use axum::{
    body::Body, extract::Request, http::Response,
    middleware::Next, response::IntoResponse, Json,
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
        if let Some(ct) = res.headers().get("content-type") {
            if ct == "plain/text" {
                let (parts, body) = res.into_parts();
                let body = String::from_utf8(
                    body.collect()
                        .await
                        .unwrap()
                        .to_bytes()
                        .into(),
                )
                .unwrap();
                let body = format!("Unknown error: {}", body);
                return (
                    parts,
                    Json(json!({
                        "error": {
                            "dev_hint": body,
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
