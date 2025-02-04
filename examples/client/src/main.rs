use axum::{
    http::{header, HeaderValue, Method},
    response::IntoResponse,
    routing::get,
    Router,
};
use cms_for_rust::{
    auth::{auth_router, init_auth},
    axum_router::collections_router,
    cms_macros::{relation, standard_collection},
    collections_editor::admin_router,
    error::CatchAll,
    migration2::run_migration,
};
use sqlx::{Pool, Sqlite};
use tower_http::{
    catch_panic::CatchPanicLayer, cors::CorsLayer,
    trace::TraceLayer,
};

#[standard_collection]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[standard_collection]
pub struct Category {
    pub title: String,
}

#[standard_collection]
pub struct Tag {
    pub title: String,
}

relation! { optional_to_many Todo Category }
relation! { many_to_many Todo Tag }

#[tokio::main]
async fn main() {
    let pool = Pool::<Sqlite>::connect("sqlite::memory:")
        .await
        .unwrap();

    run_migration(pool.clone()).await.unwrap();

    let args: Vec<String> = std::env::args().collect();
    if let Some(uns) = args.get(1) {
        // this is just en example, please use safer secret management
        if uns == "unsafe_init" {
            std::env::set_var("JWT_SALT", "secret");
        }
    }

    init_auth(pool.clone()).await.unwrap();

    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Router::new()
        .fallback(get(|| async { "404" }))
        .route("/", get(|| async { "api is working" }))
        .nest("/collectinos", collections_router())
        .nest("/admin", admin_router())
        .nest("/auth", auth_router())
        .layer(CatchPanicLayer::custom(|_| {
            CatchAll.into_response()
        }))
        .layer(
            CorsLayer::new()
                .allow_origin(
                    "http://localhost:5173"
                        .parse::<HeaderValue>()
                        .unwrap(),
                )
                .allow_methods([Method::POST])
                .allow_headers([
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                ]),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(pool.clone());

    let listner = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listner, app).await.unwrap();
}
