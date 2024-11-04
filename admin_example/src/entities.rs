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
