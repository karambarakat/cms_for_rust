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

// relations! { optional_to_many Todo Category }
// ralations! { many_to_many Todo Tag }
//
// #[entity]
// pub struct Category {
//     pub title: String,
// }
//
// #[entity]
// pub struct Tag {
//     pub title: String,
// }

