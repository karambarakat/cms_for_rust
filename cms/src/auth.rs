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
