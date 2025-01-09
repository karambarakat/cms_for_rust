pub use cms_macros;
pub use cms_macros::schema;
pub mod build_tuple;
pub mod orm;
// pub mod auth;
// pub mod axum_router;
// pub mod collections;
pub mod collections_editor;
// pub mod entities;
// pub mod migration;
// pub mod operations;
// #[cfg(test)]
// pub mod coll;
// #[cfg(test)]
// pub mod dyn_collection;
#[cfg(test)]
pub mod client_example;
pub mod queries_for_sqlx_extention;
// pub mod relations;
pub mod row_to_json_cached;
// pub mod schema_def;
pub mod testing;
pub mod tuple_index;
pub mod utils;

pub mod schema_prelude {
    // pub use crate::entities::define as entity;
    // pub use crate::relations::define as relations;
    pub use crate::schema;
}
