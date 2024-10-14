use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use sqlx::{Decode, Encode, Type};

use axum::routing::get;
use queries_for_sqlx::{
    IntoMutArguments, SupportNamedBind, SupportReturning,
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{ColumnIndex, Database, Executor, Pool};

use crate::entities::Entity;
use crate::entities::PartialEntity;
use crate::operations::delete_one::delete_one;
use crate::operations::get_all::get_all;
use crate::operations::get_one::get_one;
use crate::operations::insert_one::insert_one;
use crate::operations::update_one::update_one;

pub trait AxumRouter<S>
where
    S: Database,
{
    fn router() -> axum::Router<Pool<S>>;
}

#[derive(Debug, Default, Serialize)]
struct MyErrorInternal {
    for_dev: Option<String>,
}

pub struct MyError(StatusCode, MyErrorInternal);

impl MyError {
    pub fn for_dev(mut self, msg: String) -> Self {
        self.1.for_dev = Some(msg);
        self
    }
    #[allow(unused)]
    pub fn module_path(mut self, module_path: &str) -> Self {
        self
    }
}

pub fn error(status: StatusCode) -> MyError {
    MyError(status, MyErrorInternal::default())
}

impl IntoResponse for MyError {
    fn into_response(self) -> axum::response::Response {
        let body = json!({
            "status": self.0.as_u16(),
            "error": self.0.canonical_reason().unwrap_or_default(),
            "for_dev": self.1.for_dev.unwrap_or_default(),
        });

        (self.0, Json(body)).into_response()
    }
}

impl<S, T> AxumRouter<S> for T
where
    S: Database,
    S: SupportNamedBind + SupportReturning,
    for<'c> &'c mut S::Connection: Executor<'c, Database = S>,
    for<'r> &'r str: ColumnIndex<S::Row>,
    i64: Type<S> + for<'d> Decode<'d, S> + for<'d> Encode<'d, S>,
    T: Entity<S>
        + 'static
        + Serialize
        + for<'de> Deserialize<'de>
        + for<'q> IntoMutArguments<'q, S>
        + Send
        + Clone,
    T::Partial: PartialEntity<S>
        + for<'d> Deserialize<'d>
        + Send
        + Clone,
{
    fn router() -> Router<Pool<S>> {
        let app = Router::new()
            .route(
                "/",
                get(get_all::<S, T>).post(insert_one::<S, T>),
            )
            .route(
                "/one",
                get(get_one::<S, T>)
                    .put(update_one::<S, T>)
                    .delete(delete_one::<S, T>),
            );

        app
    }
}
