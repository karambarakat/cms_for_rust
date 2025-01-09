#![allow(unused)]

use std::{thread::sleep, time::Duration};

use cms_for_rust::axum_router::AxumRouter;
mod entites {
    use cms_for_rust::schema_prelude::*;

    schema! {
        db = "sqlx::Sqlite",
    }

    #[entity]
    pub struct Project {
        pub title: String,
        pub done: bool,
        pub description: Option<String>,
    }
}

#[tokio::main]
async fn main() {
    println!("EXECUTABLE_MANAGER:start");
    let db =
        sqlx::Pool::<sqlx::Sqlite>::connect("sqlite::memory:")
            .await
            .unwrap();

    let app = axum::Router::new()
        .nest("/", entites::Project::router())
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("localhost:3000")
        .await
        .unwrap();

    println!("EXECUTABLE_MANAGER:listening");

    axum::serve(listener, app).await.unwrap();

}
