use axum::middleware::from_fn;
use axum::{routing::post, Router};
use sqlx::{Pool, Sqlite};

use crate::auth::need_super_user;
use crate::operations::delete_one::delete_one_dynmaic as delete_one;
use crate::operations::insert_one::insert_one_dynamic as insert_one;
use crate::operations::select_many::get_all_dynamic as get_many;
use crate::operations::select_one::get_one_dynamic as get_one;
use crate::operations::update_one::update_one_dynmaic as update_one;

pub fn collections_router() -> Router<Pool<Sqlite>> {
    Router::new()
        .route("/{collection}/get_one", post(get_one))
        .route("/{collection}/get_many", post(get_many))
        .route(
            "/{collection}/insert_one",
            post(insert_one)
                .route_layer(from_fn(need_super_user)),
        )
        .route(
            "/{collection}/update_one",
            post(update_one)
                .route_layer(from_fn(need_super_user)),
        )
        .route(
            "/{collection}/delete_one",
            post(delete_one)
                .route_layer(from_fn(need_super_user)),
        )
}
