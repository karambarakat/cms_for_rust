#![allow(unused)]
use std::{
    collections::HashMap, error::Error, marker::PhantomData,
    sync::Arc,
};

use crate::{
    dynamic_schema::{SubmitDynCollection, SubmitDynRelation},
    migration2::{CreatTableSt, SubmitDynMigrate},
    queries_bridge::{SelectSt, UpdateSt},
    queries_for_sqlx_extention::{
        col_type_check_if_null, primary_key,
    },
    relations::{
        many_to_many::{ManyToMany, ManyToManyDynamic},
        optional_to_many::{
            OptionalToMany, OptionalToManyDynamic,
        },
        optional_to_many_inverse::OptionalToManyInverse,
        LinkSpec, Linked,
    },
    traits::{Collection, DynValidate, Update},
};
use inventory::submit;
use queries_for_sqlx::{
    expressions_2::{
        schema_items::col_type, schema_items_for_tupe::all,
    },
    ident_safety::PanicOnUnsafe,
    prelude::*,
    quick_query::QuickQuery,
};
use serde::{de::Visitor, Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Row, Sqlite};

impl Linked<Category> for Todo {
    type Spec = OptionalToMany;
    fn spec() -> Self::Spec {
        OptionalToMany {
            foriegn_key: "category_id".to_string(),
        }
    }
}

submit! {
    SubmitDynRelation {
        obj: || {
            Arc::new(
                OptionalToManyDynamic::<Todo, Category>::new()
            )
        }
    }
}

impl Linked<Todo> for Category {
    type Spec = OptionalToManyInverse;
    fn spec() -> Self::Spec {
        OptionalToManyInverse
    }
}

impl Linked<Tag> for Todo {
    type Spec = ManyToMany;
    fn spec() -> Self::Spec {
        ManyToMany {
            conjuction_table: format!(
                "{}{}",
                Todo::table_name(),
                Tag::table_name(),
            ),
            // this should be the inverse
            base_id: format!(
                "{}_id",
                Todo::table_name().to_lowercase()
            ),
            destination_id: format!(
                "{}_id",
                Tag::table_name().to_lowercase()
            ),
        }
    }
}

submit! {
    SubmitDynRelation {
        obj: || {
            Arc::new(
                ManyToManyDynamic::<Todo, Tag>::new()
            )
        }
    }
}

submit! {
    SubmitDynRelation {
        obj: || {
            Arc::new(
                ManyToManyDynamic::<Tag, Todo>::new()
            )
        }
    }
}

impl Linked<Todo> for Tag {
    type Spec = ManyToMany;
    fn spec() -> Self::Spec {
        ManyToMany {
            conjuction_table: format!(
                "{}{}",
                Todo::table_name(),
                Tag::table_name(),
            ),
            // this should be the inverse
            base_id: format!(
                "{}_id",
                Tag::table_name().to_lowercase()
            ),
            destination_id: format!(
                "{}_id",
                Todo::table_name().to_lowercase()
            ),
        }
    }
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Serialize,
    Deserialize,
)]
pub struct Todo {
    // #[field(Regex(r#"^.{1,100}$"#))]
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

submit! {SubmitDynCollection {
    obj: || Box::new(PhantomData::<Todo>)
}}
submit!(SubmitDynMigrate {
    obj: || Box::new(PhantomData::<Todo>)
});

#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Serialize,
    Deserialize,
)]
pub struct Partial {
    pub title: Update<String>,
    pub done: Update<bool>,
    pub description: Update<Option<String>>,
}

impl Collection<Sqlite> for Todo {
    fn on_migrate(stmt: &mut CreatTableSt<Sqlite>) {
        stmt.column("id", primary_key::<Sqlite>());
        stmt.column("title", col_type_check_if_null::<String>());
        stmt.column("done", col_type_check_if_null::<bool>());
        stmt.column(
            "description",
            col_type_check_if_null::<String>(),
        );
    }
    fn members() -> &'static [&'static str] {
        &["title", "done", "description"]
    }
    fn members_scoped() -> &'static [&'static str] {
        &["todo_title", "todo_done", "todo_description"]
    }
    type PartailCollection = Partial;
    fn on_update(
        stmt: &mut UpdateSt<Sqlite>,
        this: Self::PartailCollection,
    ) -> Result<(), std::string::String> {
        if let Update::set(val) = this.title {
            stmt.set("title".to_string(), {
                let mut val = val;

                // Regex(r#"^.{1,100}$"#).modify(&mut val)?;

                val
            });
        }
        if let Update::set(val) = this.done {
            stmt.set("done".to_string(), val);
        }

        if let Update::set(val) = this.description {
            stmt.set("description".to_string(), val);
        }

        Ok(())
    }
    // fn on_update_ref_mod(
    //     this: Value,
    //     stmt: &mut UpdateSt,
    //     mods: &HashMap<
    //         String,
    //         Vec<Box<dyn DynValidate + Send + Sync>>,
    //     >,
    // ) {
    //     if let Some(val) = this.as_object().unwrap().get("title")
    //     {
    //         stmt.set("title".to_string(), {
    //             let mut val = val.clone();
    //
    //             let hi = mods
    //                 .get("title")
    //                 .unwrap()
    //                 .iter()
    //                 .try_for_each(|modifier| {
    //                     modifier.validate_on_update(&mut val)
    //                 });
    //
    //             val
    //         });
    //     }
    //
    //     if let Some(val) = this.as_object().unwrap().get("done")
    //     {
    //         stmt.set("done".to_string(), {
    //             let mut val = val.clone();
    //
    //             mods.get("done")
    //                 .unwrap()
    //                 .iter()
    //                 .try_for_each(|modifier| {
    //                     modifier.validate_on_update(&mut val)
    //                 })
    //                 .unwrap();
    //
    //             val
    //         });
    //     }
    //
    //     if let Some(val) =
    //         this.as_object().unwrap().get("description")
    //     {
    //         stmt.set("description".to_string(), {
    //             let mut val = val.clone();
    //
    //             mods.get("description")
    //                 .unwrap()
    //                 .iter()
    //                 .try_for_each(|modifier| {
    //                     modifier.validate_on_update(&mut val)
    //                 })
    //                 .unwrap();
    //
    //             val
    //         });
    //     }
    // }
    //
    // fn get_all_modifiers(
    // ) -> HashMap<String, Vec<Box<dyn DynValidate + Send + Sync>>>
    // {
    //     let mut map = HashMap::new();
    //     map.insert(
    //         "title".to_string(),
    //         vec![
    //             // Regex(r#"^.{1,100}$"#).to_dyn_modifier(),
    //         ],
    //     );
    //
    //     map.insert("done".to_string(), vec![]);
    //     map.insert("description".to_string(), vec![]);
    //     map
    // }
    fn table_name() -> &'static str {
        "Todo"
    }
    fn on_select(stmt: &mut SelectSt<Sqlite>) {
        stmt.select("title");
        stmt.select("done");
        stmt.select("description");
    }
    fn on_insert(
        self,
        stmt: &mut stmt::InsertStOne<'_, sqlx::Sqlite>,
    ) -> Result<(), String> {
        stmt.insert("title".to_owned(), {
            // run mods

            let mut val = self.title;

            // Regex(r#"^.{1,100}$"#).modify(&mut val)?;

            val
        });
        stmt.insert("done".to_owned(), self.done);
        stmt.insert("description".to_owned(), self.description);
        Ok(())
    }
    // fn on_insert_returning() -> Vec<&'static str> {
    //     vec!["title", "done", "description"]
    // }
    // fn on_insert_ref_mod(
    //     this: Value,
    //     stmt: &mut stmt::InsertStOne<'_, sqlx::Sqlite>,
    //     mods: &HashMap<
    //         String,
    //         Vec<Box<dyn DynValidate + Send + Sync>>,
    //     >,
    // ) {
    //     stmt.insert("title".to_owned(), {
    //         // run mods
    //
    //         let mut val = this
    //             .as_object()
    //             .unwrap()
    //             .get("title")
    //             .unwrap()
    //             .clone();
    //
    //         mods.get("title")
    //             .unwrap()
    //             .iter()
    //             .try_for_each(|modifier| {
    //                 modifier.validate_on_insert(&mut val)
    //             })
    //             .unwrap();
    //
    //         val
    //     });
    //     stmt.insert(
    //         "done".to_owned(),
    //         this.as_object()
    //             .unwrap()
    //             .get("done")
    //             .unwrap()
    //             .clone(),
    //     );
    //     stmt.insert(
    //         "description".to_owned(),
    //         this.as_object()
    //             .unwrap()
    //             .get("description")
    //             .unwrap()
    //             .clone(),
    //     );
    // }
    fn from_row_scoped(row: &sqlx::sqlite::SqliteRow) -> Self {
        Self {
            // run all mods
            title: {
                let title = row.get("title");

                // Regex(r#"^.{1,100}$"#).modify_on_select(&mut title)?;

                title
            },
            done: row.get("done"),
            description: row.get("description"),
        }
    }
    fn from_row_noscope(row: &sqlx::sqlite::SqliteRow) -> Self {
        Self {
            // run all mods
            title: {
                let title = row.get("title");

                // Regex(r#"^.{1,100}$"#).modify_on_select(&mut title)?;

                title
            },
            done: row.get("done"),
            description: row.get("description"),
        }
    }

    // fn on_get_no_mods(
    //     row: &mut sqlx::sqlite::SqliteRow,
    // ) -> Self {
    //     Self {
    //         title: row.get("title"),
    //         done: row.get("done"),
    //         description: row.get("description"),
    //     }
    // }
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Serialize,
    Deserialize,
)]
pub struct Category {
    pub cat_title: String,
}

submit! {SubmitDynCollection {
    obj: || Box::new(PhantomData::<Category>)
}}
submit!(SubmitDynMigrate {
    obj: || Box::new(PhantomData::<Category>)
});

#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Serialize,
    Deserialize,
)]
pub struct PartialCategory {
    pub cat_title: Update<String>,
}

impl Collection<Sqlite> for Category {
    type PartailCollection = PartialCategory;
    fn on_migrate(stmt: &mut CreatTableSt<Sqlite>) {
        stmt.column("id", primary_key::<Sqlite>());
        stmt.column(
            "cat_title",
            col_type_check_if_null::<String>(),
        );
    }
    fn members() -> &'static [&'static str] {
        &["cat_title"]
    }
    fn members_scoped() -> &'static [&'static str] {
        &["cat_cat_title"]
    }
    fn on_update(
        stmt: &mut UpdateSt<Sqlite>,
        this: Self::PartailCollection,
    ) -> Result<(), String> {
        if let Update::set(val) = this.cat_title {
            stmt.set("cat_title".to_string(), val);
        }
        Ok(())
    }
    // fn on_update_ref_mod(
    //     this: Value,
    //     stmt: &mut UpdateSt,
    //     mods: &HashMap<
    //         String,
    //         Vec<Box<dyn DynValidate + Send + Sync>>,
    //     >,
    // ) {
    //     if let Some(val) = this.as_object().unwrap().get("title")
    //     {
    //         stmt.set("cat_title".to_string(), {
    //             let mut val = val.clone();
    //
    //             mods.get("cat_title")
    //                 .unwrap()
    //                 .iter()
    //                 .try_for_each(|modifier| {
    //                     modifier.validate_on_update(&mut val)
    //                 })
    //                 .unwrap();
    //
    //             val
    //         });
    //     }
    // }
    //
    // fn get_all_modifiers(
    // ) -> HashMap<String, Vec<Box<dyn DynValidate + Send + Sync>>>
    // {
    //     let mut map = HashMap::new();
    //     map.insert("cat_title".to_string(), vec![]);
    //
    //     map
    // }
    fn table_name() -> &'static str {
        "Category"
    }
    fn on_select(stmt: &mut SelectSt<Sqlite>) {
        stmt.select_aliased(
            "Category",
            "cat_title",
            "cat_cat_title",
        );
    }
    fn on_insert(
        self,
        stmt: &mut stmt::InsertStOne<'_, sqlx::Sqlite>,
    ) -> Result<(), String> {
        stmt.insert("cat_title".to_owned(), self.cat_title);
        Ok(())
    }
    // fn on_insert_returning() -> Vec<&'static str> {
    //     vec!["cat_title"]
    // }
    // fn on_insert_ref_mod(
    //     this: Value,
    //     stmt: &mut stmt::InsertStOne<'_, sqlx::Sqlite>,
    //     mods: &HashMap<
    //         String,
    //         Vec<Box<dyn DynValidate + Send + Sync>>,
    //     >,
    // ) {
    //     stmt.insert("cat_title".to_owned(), {
    //         let mut val = this
    //             .as_object()
    //             .unwrap()
    //             .get("cat_title")
    //             .unwrap()
    //             .clone();
    //
    //         mods.get("cat_title")
    //             .unwrap()
    //             .iter()
    //             .try_for_each(|modifier| {
    //                 modifier.validate_on_insert(&mut val)
    //             })
    //             .unwrap();
    //
    //         val
    //     });
    // }
    fn from_row_scoped(row: &sqlx::sqlite::SqliteRow) -> Self {
        Self {
            cat_title: row.try_get("cat_cat_title").unwrap(),
        }
    }
    fn from_row_noscope(row: &sqlx::sqlite::SqliteRow) -> Self {
        Self {
            cat_title: row.try_get("cat_title").unwrap(),
        }
    }
    // fn on_get_no_mods(
    //     row: &mut sqlx::sqlite::SqliteRow,
    // ) -> Self {
    //     Self {
    //         cat_title: row.try_get("cat_cat_title").unwrap(),
    //     }
    // }
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Serialize,
    Deserialize,
)]

pub struct Tag {
    pub tag_title: String,
}

submit! {SubmitDynCollection {
    obj: || Box::new(PhantomData::<Tag>)
}}
submit!(SubmitDynMigrate {
    obj: || Box::new(PhantomData::<Tag>)
});

#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Serialize,
    Deserialize,
)]
pub struct PartialTag {
    pub tag_title: Update<String>,
}

impl Collection<Sqlite> for Tag {
    type PartailCollection = PartialTag;
    fn on_migrate(stmt: &mut CreatTableSt<Sqlite>) {
        stmt.column("id", primary_key::<Sqlite>());
        stmt.column(
            "tag_title",
            col_type_check_if_null::<String>(),
        );
    }
    fn members() -> &'static [&'static str] {
        &["tag_title"]
    }

    fn members_scoped() -> &'static [&'static str] {
        &["tag_tag_title"]
    }

    fn on_update(
        stmt: &mut UpdateSt<Sqlite>,
        this: Self::PartailCollection,
    ) -> Result<(), String> {
        if let Update::set(val) = this.tag_title {
            stmt.set("tag_title".to_string(), val);
        }

        Ok(())
    }

    // fn on_update_ref_mod(
    //     this: Value,
    //     stmt: &mut UpdateSt,
    //     mods: &HashMap<
    //         String,
    //         Vec<Box<dyn DynValidate + Send + Sync>>,
    //     >,
    // ) {
    //     if let Some(val) = this.as_object().unwrap().get("title")
    //     {
    //         stmt.set("tag_title".to_string(), {
    //             let mut val = val.clone();
    //
    //             mods.get("tag_title")
    //                 .unwrap()
    //                 .iter()
    //                 .try_for_each(|modifier| {
    //                     modifier.validate_on_update(&mut val)
    //                 })
    //                 .unwrap();
    //
    //             val
    //         });
    //     }
    // }
    //
    // fn get_all_modifiers(
    // ) -> HashMap<String, Vec<Box<dyn DynValidate + Send + Sync>>>
    // {
    //     let mut map = HashMap::new();
    //     map.insert("tag_title".to_string(), vec![]);
    //
    //     map
    // }
    fn table_name() -> &'static str {
        "Tag"
    }
    fn on_select(stmt: &mut SelectSt<Sqlite>) {
        stmt.select_aliased("Tag", "tag_title", "tag_tag_title");
    }
    fn on_insert(
        self,
        stmt: &mut stmt::InsertStOne<'_, sqlx::Sqlite>,
    ) -> Result<(), String> {
        stmt.insert("title".to_owned(), self.tag_title);
        Ok(())
    }
    // fn on_insert_returning() -> Vec<&'static str> {
    //     vec!["tag_title"]
    // }
    // fn on_insert_ref_mod(
    //     this: Value,
    //     stmt: &mut stmt::InsertStOne<'_, sqlx::Sqlite>,
    //     mods: &HashMap<
    //         String,
    //         Vec<Box<dyn DynValidate + Send + Sync>>,
    //     >,
    // ) {
    //     stmt.insert("tag_title".to_owned(), {
    //         let mut val = this
    //             .as_object()
    //             .unwrap()
    //             .get("tag_title")
    //             .unwrap()
    //             .clone();
    //
    //         mods.get("tag_title")
    //             .unwrap()
    //             .iter()
    //             .try_for_each(|modifier| {
    //                 modifier.validate_on_insert(&mut val)
    //             })
    //             .unwrap();
    //
    //         val
    //     });
    // }
    fn from_row_scoped(row: &sqlx::sqlite::SqliteRow) -> Self {
        Self {
            tag_title: row.try_get("tag_tag_title").unwrap(),
        }
    }
    fn from_row_noscope(row: &sqlx::sqlite::SqliteRow) -> Self {
        Self {
            tag_title: row.try_get("tag_title").unwrap(),
        }
    }
    // fn on_get_no_mods(
    //     row: &mut sqlx::sqlite::SqliteRow,
    // ) -> Self {
    //     Self {
    //         tag_title: row.try_get("tag_title").unwrap(),
    //     }
    // }
}
