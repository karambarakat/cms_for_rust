use axum::{
    handler::Handler,
    http::{header, HeaderValue, Method},
    routing::get,
    Router,
};
use cms_for_rust::{
    auth::{auth_router, create_super_user_if_not_exist}, axum_router::collections_router, cms_macros::{relation, standard_collection}, collections_editor::admin_router, initialization::verify_initialization, migration2::run_migration
};
use sqlx::{Pool, Sqlite};
use tower_http::{
    cors::{self, CorsLayer},
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

    // how to parse input arg?
    let args: Vec<String> = std::env::args().collect();
    if let Some(uns) = args.get(1) {
        if uns == "unsafe_init" {
            // this is just en example, please use safer secret management
            std::env::set_var("JWT_SALT", "secret");
        }
    }

    verify_initialization().expect("the app may not be initialized correctly");

    if let Some(token) =
        create_super_user_if_not_exist(pool.clone()).await
    {
        let be = "http://localhost:3000";
        let fe = "http://localhost:5173";
        println!("Looks like you have no super user");
        print!("Create your first at ");
        println!(
            "{fe}/auth/init?token={token}&backend_url={be}"
        );
        println!(
            "Or initiate different database at the same page"
        );
    }

    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Router::new()
        .route("/", get(|| async { "api is working" }))
        .nest("/collectinos", collections_router())
        .nest("/admin", admin_router())
        .nest("/auth", auth_router())
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
