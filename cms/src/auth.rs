mod email_password;
mod ijwt;
mod other;
use axum::{
    http::StatusCode, middleware::from_fn, routing::post, Router,
};
pub use other::*;

use sqlx::{Pool, Sqlite};

pub fn auth_router() -> Router<Pool<Sqlite>> {
    Router::new()
        .nest(
            "/init",
            Router::new()
                .route("/sign_in_first", post(sign_in_existing))
                .route(
                    "/set_up_db",
                    post(|| async {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "todo",
                        )
                    }),
                )
                .layer((
                    from_fn(need_super_user),
                    from_fn(other::can_init),
                )),
        )
        .route(
            "/sign_in_invited",
            post(sign_in_existing)
                .route_layer(from_fn(need_super_user)),
        )
        .route(
            "/login",
            post(login)
                .route_layer(from_fn(extend_for_authinticated)),
        )
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Method, Request},
    };
    use http_body_util::BodyExt;
    use serde_json::json;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn test_auth_router() {
        let pool = sqlx::Pool::connect("sqlite::memory:")
            .await
            .unwrap();
        std::env::set_var("JWT_SALT", "test");

        let token = create_super_user_if_not_exist_and_return_init_token(
            pool.clone(),
        )
        .await
        .expect("new db therfore token should be generated");

        let app = auth_router().with_state(pool.clone());

        let res = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .header(
                        "authorization",
                        format!("Bearer {}", token),
                    )
                    .uri("/init/sign_in_first")
                    .body(Body::from(json!({}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // res body as text
        let res_text =
            res.into_body().collect().await.unwrap().to_bytes();
        let res_text =
            String::from_utf8(res_text.into()).unwrap();

        panic!("{:?}", res_text);

        if res.status().is_success() {
            panic!("failed to sign in first");
        }

        //
    }
}
