pub use cms_macros;
pub use cms_macros::schema;

pub mod tuple_index;
pub mod axum_router;
pub mod entities;
pub mod migration;
pub mod operations;
pub mod queries_for_sqlx_extention;
pub mod relations;
pub mod row_to_json_cached;
pub mod utils;

pub mod schema_prelude {
    pub use crate::entities::define as entity;
    pub use crate::relations::define as relations;
    pub use crate::schema;
}
