use std::{
    collections::HashMap, error::Error, marker::PhantomData,
};

use queries_for_sqlx::{ident_safety::PanicOnUnsafe, prelude::stmt, quick_query::QuickQuery};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};

use crate::collections::{Collection, DynModifier, Modifier};

pub trait DynCollection {
    fn table_name(&self) -> &str;
    fn table_name_s(&self) -> &'static str;
    fn on_select(
        &self,
        stmt: &mut stmt::SelectSt<sqlx::Sqlite, QuickQuery, PanicOnUnsafe>,
    );
    fn on_get(
        &self,
        row: &mut sqlx::sqlite::SqliteRow,
    ) -> Result<Value, Box<dyn Error>>;
    fn on_insert(
        &self,
        stmt: &mut stmt::InsertStOne<'_, sqlx::Sqlite>,
        this: Value,
    );
    fn on_get_insert(
        &self,
        row: &mut sqlx::sqlite::SqliteRow,
    ) -> Value;
    fn on_insert_returning(&self) -> Vec<String>;
    fn on_update(
        &self,
        stmt: &mut stmt::UpdateSt<sqlx::Sqlite, QuickQuery>,
        this: Value,
    );
}

impl<T> DynCollection for TrivialCollection<T>
where
    T: Collection + Serialize + DeserializeOwned,
    T::PartailCollection: DeserializeOwned,
{
    fn on_update(
        &self,
        stmt: &mut stmt::UpdateSt<sqlx::Sqlite, QuickQuery>,
        this: Value,
    ) {
        T::on_update_ref_mod(this, stmt, &self.mods);
    }
    fn table_name(&self) -> &str {
        self.table_name.as_str()
    }
    fn table_name_s(&self) -> &'static str {
        T::table_name1()
    }
    fn on_select(
        &self,
        stmt: &mut stmt::SelectSt<sqlx::Sqlite, QuickQuery, PanicOnUnsafe>,
    ) {
        T::on_select1(stmt);
    }
    fn on_insert_returning(&self) -> Vec<String> {
        self.returns.clone()
    }
    fn on_insert(
        &self,
        stmt: &mut stmt::InsertStOne<'_, sqlx::Sqlite>,
        data: Value,
    ) {
        T::on_insert_ref_mod(data, stmt, &self.mods);
    }
    fn on_get_insert(
        &self,
        row: &mut sqlx::sqlite::SqliteRow,
    ) -> Value {
        let hi = T::on_get_no_mods(row);
        let mut hi = serde_json::to_value(hi).unwrap();

        hi.into()
    }
    fn on_get(
        &self,
        row: &mut sqlx::sqlite::SqliteRow,
    ) -> Result<Value, Box<dyn Error>> {
        let hi = T::on_get_no_mods(row);
        let mut hi = serde_json::to_value(hi).unwrap();

        let obj = hi
            .as_object_mut()
            .expect("collections has to be structs");

        for (key, mods) in self.mods.iter() {
            for modifier in mods {
                modifier.modify_on_select(&mut obj[key])?;
            }
        }

        Ok(hi.into())
    }
}

pub struct TrivialCollection<T> {
    pub table_name: String,
    // why Sync: ExecuteNoCache::fetch* takes closure and I want to get a ref for Collection
    // why Send: in get_one we work on Collection and use .await
    pub mods:
        HashMap<String, Vec<Box<dyn DynModifier + Sync + Send>>>,
    pub returns: Vec<String>,
    _phantom: PhantomData<T>,
}

impl<T: Collection> TrivialCollection<T> {
    pub fn new() -> Self {
        Self {
            table_name: T::table_name1().to_string(),
            mods: T::get_all_modifiers(),
            returns: T::on_insert_returning()
                .iter()
                .map(|e| e.to_string())
                .collect(),
            _phantom: PhantomData,
        }
    }
}
