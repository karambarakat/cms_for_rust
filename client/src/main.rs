use axum::Router;
use cms_for_rust::{
    auth::auth_router,
    axum_router::collections_router,
    cms_macros::{relation, standard_collection},
    collections_editor::admin_router,
    migration2::run_migration,
};
use sqlx::{Pool, Sqlite};

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
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let pool = Pool::<Sqlite>::connect("sqlite::memory:")
        .await
        .unwrap();

    run_migration(pool.clone()).await.unwrap();

    let app = Router::new()
        .nest("/collectinos", collections_router())
        .nest("/admin", admin_router())
        .nest("/auth", auth_router())
        .with_state(pool.clone());

    let listner = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listner, app).await.unwrap();
}
