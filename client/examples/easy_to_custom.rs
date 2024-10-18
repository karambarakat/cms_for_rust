use cms_for_rust::cms_macros::relations;
use cms_for_rust::cms_macros::service as entity;

use sqlx::Sqlite;

#[allow(non_camel_case_types)]
type CMS_DB = Sqlite;

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
relations! { optional_to_many Todo Category }

#[cfg(test)]
mod test {
    use super::*;
    use axum::{extract::State, response::IntoResponse};
    use cms_for_rust::{
        entities::Entity,
        row_to_json_cached::{self, sqlite_row},
    };
    use queries_for_sqlx::{
        prelude::*, quick_query::QuickQuery, string_query,
        SupportNamedBind,
    };
    use serde_json::{json, Value};
    use sqlx::{Database, Pool};
    use stmt::SelectSt;

    fn select_st<'q, S, E>() -> SelectSt<S, QuickQuery<'q>>
    where
        E: Entity<S>,
        S: SupportNamedBind + Database,
    {
        stmt::SelectSt::<S, QuickQuery>::init(E::table_name())
    }

    async fn hi(db: State<Pool<CMS_DB>>) -> Vec<Value> {
        let mut st = select_st::<_, Todo>();

        st.select(all_columns());

        st.fetch_all(&db.0, row_to_json_cached::sqlite_row())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn main_test() {
        let pool =
            sqlx::Pool::<Sqlite>::connect("sqlite::memory:")
                .await
                .unwrap();

        string_query::StringQuery{sql: "CREATE TABLE todo (title TEXT, done BOOLEAN, description TEXT);
INSERT INTO todo (title, done, description) VALUES ('hi', 1, 'hello'), ('bye', 0, 'goodbye');
".to_owned(), input: ()}
            .execute(&pool)
            .await
            .unwrap();

        let res: Value = hi(State(pool)).await.into();

        assert_eq!(
            res,
            json!([
                {"title": "hi", "done": true, "description": "hello"},
                {"title": "bye", "done": false, "description": "goodbye"}
            ])
        );
    }
}

fn main() {}
