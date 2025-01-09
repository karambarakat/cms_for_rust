#![allow(unused)]
use sqlx::sqlite::SqliteRow;
use std::{collections::HashMap, error::Error};

use serde_json::Value;
use sqlx::Sqlite;

use queries_for_sqlx::{
    self, ident_safety::PanicOnUnsafe, prelude::*,
    quick_query::QuickQuery,
};

pub mod error {
    use axum::{http::StatusCode, response::IntoResponse, Json};
    use serde::Serialize;
    use serde_json::json;

    #[derive(Debug, Default, Serialize)]
    struct ErrorInternal {
        for_dev: Option<String>,
    }

    pub struct CollectionError(StatusCode, ErrorInternal);

    impl CollectionError {
        pub fn for_dev(mut self, msg: String) -> Self {
            self.1.for_dev = Some(msg);
            self
        }
    }

    impl IntoResponse for CollectionError {
        fn into_response(self) -> axum::response::Response {
            let body = json!({
                "status": self.0.as_u16(),
                "error": self.0.canonical_reason().unwrap_or_default(),
                "for_dev": self.1.for_dev.unwrap_or_default(),
            });

            (self.0, Json(body)).into_response()
        }
    }

    pub fn to_refactor(code: StatusCode) -> CollectionError {
        todo!()
    }

    pub fn not_found(id: i32) -> CollectionError {
        CollectionError(
            StatusCode::NOT_FOUND,
            ErrorInternal::default(),
        )
    }
    pub fn missing_id_in_query() -> CollectionError {
        CollectionError(
            StatusCode::BAD_REQUEST,
            ErrorInternal::default(),
        )
    }
}

pub trait Modifier {
    type Value;
    fn modify_on_select(&self, value: &mut Self::Value) {}
    fn modify_on_insert(
        &self,
        value: &mut Self::Value,
    ) -> Result<(), String> {
        Ok(())
    }
    fn modify_on_update(
        &self,
        value: &mut Self::Value,
    ) -> Result<(), String> {
        Ok(())
    }
    fn modify_on_delete(
        &self,
        value: &mut Self::Value,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

pub trait DynModifier {
    fn modify_on_select(
        &self,
        value: &mut Value,
    ) -> Result<(), Box<dyn Error>>;
    fn modify_on_insert(
        &self,
        value: &mut Value,
    ) -> Result<(), Box<dyn Error>>;
    fn modify_on_update(
        &self,
        value: &mut Value,
    ) -> Result<(), Box<dyn Error>>;
    fn modify_on_delete(
        &self,
        value: &mut Value,
    ) -> Result<(), Box<dyn Error>>;
}

pub trait Collection: Sized + Send + Sync {
    type PartailCollection;
    fn on_update(
        stmt: &mut stmt::UpdateSt<Sqlite, QuickQuery>,
        this: Self::PartailCollection,
    );
    fn on_update_ref_mod(
        this: Value,
        stmt: &mut stmt::UpdateSt<Sqlite, QuickQuery>,
        mods: &HashMap<
            String,
            Vec<Box<dyn DynModifier + Send + Sync>>,
        >,
    );

    // done
    fn table_name1() -> &'static str;
    // done
    fn on_select1(
        stmt: &mut stmt::SelectSt<
            Sqlite,
            QuickQuery,
            PanicOnUnsafe,
        >,
    );
    // done
    fn on_get1(row: &SqliteRow) -> Self;
    // why I modifing on_get at the first place??
    fn on_get_no_mods(row: &mut SqliteRow) -> Self;
    fn on_insert(self, stmt: &mut stmt::InsertStOne<'_, Sqlite>);
    fn on_insert_returning() -> Vec<&'static str>;
    fn on_insert_ref_mod(
        this: Value,
        stmt: &mut stmt::InsertStOne<'_, Sqlite>,
        mods: &HashMap<
            String,
            Vec<Box<dyn DynModifier + Send + Sync>>,
        >,
    );

    // why Send+Sync: see comments on TrivialCollection
    fn get_all_modifiers(
    ) -> HashMap<String, Vec<Box<dyn DynModifier + Send + Sync>>>;
}
