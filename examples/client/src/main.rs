use axum::{
    http::{header, HeaderName, HeaderValue, Method},
    middleware::from_fn,
    response::IntoResponse,
    routing::{self, get},
    Router,
};
use cms_for_rust::{
    uniform_response_layer::uniform_response_middleware,
    auth::{auth_router, init_auth},
    axum_router::collections_router,
    cms_macros::{relation, standard_collection},
    collections_editor::admin_router,
    error::{ClientError, PanicError},
    migration2::run_migration,
    schema_info::schema_router,
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
        .fallback(routing::any(|| async {
            ClientError::endpoint_not_found()
        }))
        .route("/", get(|| async { "api is working" }))
        .nest("/collection", collections_router())
        .nest("/admin", admin_router())
        .nest("/auth", auth_router())
        .with_state(pool.clone())
        .nest("/schema", schema_router())
        .layer(from_fn(uniform_response_middleware))
        .layer(CatchPanicLayer::custom(|_| {
            // todo
            // 1. send this error to developers
            // 2. make an id out of it (hash it or add it to a static) and send it to user
            //
            // 3. advanced: have a different layer that integrate with std::panic::set_hook
            //  3.1. get the location of the panic

            // let _err: &str = if let Some(e) =
            //     ty.downcast_ref::<String>()
            // {
            //     Some(e.as_str())
            // } else if let Some(e) = ty.downcast_ref::<&str>()
            // {
            //     Some(*e)
            // } else {
            //     None
            // };
            PanicError.into_response()
        }))
        .layer(
            CorsLayer::new()
                .allow_origin(
                    "http://localhost:5173"
                        .parse::<HeaderValue>()
                        .unwrap(),
                )
                .allow_methods([Method::POST])
                .expose_headers([HeaderName::from_bytes(
                    b"X-Cms-Token",
                )
                .unwrap()])
                .allow_headers([
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                ]),
        )
        .layer(TraceLayer::new_for_http());

    let listner = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listner, app).await.unwrap();
}
