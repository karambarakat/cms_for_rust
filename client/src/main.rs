use axum::{http::StatusCode, routing::any, Router};
use cms_for_rust::{
    auth,
    axum_router::AxumRouter,
    collections, collections_editor,
    entities::define as entity,
    hmr::{self, dyn_entities_router, DynEntitiesRouter},
    relations::define as relations,
    schema, schema_def,
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

#[tokio::main]
async fn main() {
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let pool =
        sqlx::Pool::<sqlx::Sqlite>::connect("sqlite::memory:")
            .await
            .unwrap();

    cms_for_rust::migration::migrate(pool.clone()).await;

    if cfg!(feature = "HMR") {
        let app = "nest dyn_router::<(Todo, ";
    } else {
        let app = "nest router";
    }

    let app = Router::new()
        .nest("/todo", Todo::router())
        .nest("/collections", collections::router())
        .nest(
            "/collections_editor",
            collections_editor::router(pool.clone()),
        )
        .with_state(pool.clone())
        .route_layer(auth::layer());

    let listner = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listner, app).await.unwrap();
}
