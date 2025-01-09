use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use queries_for_sqlx::ident_safety::PanicOnUnsafe;
use queries_for_sqlx::prelude::*;
use queries_for_sqlx::quick_query::QuickQuery;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Map;
use serde_json::Value;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use sqlx::Sqlite;

use crate::collections::Collection;

use super::dynamic::DynCollection;
use super::rel_trait::DynRelation;
use super::rel_trait::*;

pub struct OptionalToMany<O, M> {
    pub to: String,
    pub local_fk: String,
    pub entity: Box<dyn DynCollection + Send + Sync>,
    pub _ent: PhantomData<(O, M)>,
}

impl<O, M> OptionalToMany<O, M> {
    pub fn new(to: String) -> Self {
        Self {
            to: to.clone(),
            entity: todo!(),
            local_fk: format!("{}_id", to),
            _ent: PhantomData,
        }
    }
}

pub struct OMGenOne<O, M> {
    pub origin: Arc<OptionalToMany<O, M>>,
    pub output: HashMap<i64, Value>,
    pub from_many: bool,
}

#[derive(Deserialize)]
enum InsertIn {
    LinkId(i64),
}

pub struct OMGenOneInsert<O, M> {
    pub origin: Arc<OptionalToMany<O, M>>,
    pub input: InsertIn,
}

#[derive(Deserialize)]
enum UpdateIn {
    UpdateId(i64),
}

pub struct OMGenOneUpdate<O, M> {
    pub origin: Arc<OptionalToMany<O, M>>,
    pub input: UpdateIn,
}

impl<O, M> OnUpdateRelation for OMGenOneUpdate<O, M>
where
    O: 'static,
    M: 'static + Collection + Serialize,
{
    fn key(&self) -> &str {
        &self.origin.to
    }
    fn on_update(
        &mut self,
        stmt: &mut stmt::UpdateSt<Sqlite, QuickQuery>,
    ) {
        let UpdateIn::UpdateId(id) = self.input;
        stmt.set(
            self.origin.local_fk.clone().leak(),
            move || id,
        );
    }
    fn take(&mut self) -> Value {
        let UpdateIn::UpdateId(id) = self.input;
        json!({"id": id})
    }
}
impl<O, M> OnInsertRelation for OMGenOneInsert<O, M>
where
    O: 'static,
    M: 'static + Collection + Serialize,
{
    fn key(&self) -> &str {
        &self.origin.to
    }
    fn on_insert(
        &mut self,
        stmt: &mut stmt::InsertStOne<'_, Sqlite>,
    ) {
        let InsertIn::LinkId(id) = self.input;
        stmt.insert(self.origin.local_fk.clone().leak(), id)
    }
    fn take(&mut self) -> Value {
        let InsertIn::LinkId(id) = self.input;
        json!({"id": id})
    }
}

impl<O, M> OnGetRelation for OMGenOne<O, M>
where
    O: 'static,
    M: 'static + Collection + Serialize,
{
    fn key(&self) -> &str {
        &self.origin.to
    }
    fn on_select(
        &mut self,
        stmt: &mut stmt::SelectSt<
            Sqlite,
            QuickQuery,
            PanicOnUnsafe,
        >,
    ) {
        stmt.join(Join {
            ty: "LEFT JOIN",
            on_table: self.origin.to.clone(),
            on_column: "id".to_string(),
            local_column: self.origin.local_fk.clone(),
        });

        // if id needed
        stmt.select(col(self.origin.local_fk.clone()));

        // if populate
        self.origin.entity.on_select(stmt);
    }
    fn from_row(&mut self, row: &mut SqliteRow) {
        let id: Option<i64> =
            row.get(self.origin.local_fk.as_str());
        if let Some(id) = id {
            let value = self.origin.entity.on_get(row).unwrap();
            let value = serde_json::to_value(value).unwrap();
            self.output.insert(id, value);
        }
    }
    fn take(&mut self, id: i64) -> Value {
        if self.from_many {
            return self
                .output
                .remove(&id)
                .unwrap_or(Value::Null);
        } else {
            let v: Vec<_> = self.output.drain().collect();
            return v
                .get(0)
                .map(|(_, val)| val.clone())
                .unwrap_or(Value::Null);
        }
    }
}

impl<O, M> DynRelation for OptionalToMany<O, M>
where
    O: 'static + Send + Sync,
    M: 'static + Send + Sync + Collection + Serialize,
{
    fn init_get(
        self: Arc<Self>,
        op: Operation,
        to: &str,
        velue: Value,
    ) -> Result<Box<dyn OnGetRelation + Send + Sync>, String>
    {
        if to != self.to {
            return Err(format!(
                "relation for \"{}\" not found",
                to
            ));
        }
        let mut output: HashMap<i64, Value> = Default::default();
        Ok(Box::new(
            ImplOnGetRelation {
                this: self.clone(),
                data: output,
                key: self.to.clone(),
                on_select: |this, _, stmt| {
                    stmt.join(Join {
                        ty: "LEFT JOIN",
                        on_table: this.to.to_string(),
                        on_column: "id".to_string(),
                        local_column: this.local_fk.to_string(),
                    });

                    // if id needed
                    stmt.select(col(this.local_fk.to_string()));

                    // if populate
                    this.entity.on_select(stmt);
                },
                from_row: |this, output, row| {
                    let id: Option<i64> =
                        row.get(this.local_fk.as_str());
                    if let Some(id) = id {
                        let value =
                            this.entity.on_get(row).unwrap();
                        let value =
                            serde_json::to_value(value).unwrap();
                        output.insert(id, value);
                    }
                },
                sub_op: |this, _, i| todo!(),
                take: move |this, output, id| {
                    if let Operation::SelectMany = op.clone() {
                        return output
                            .remove(&id)
                            .unwrap_or(Value::Null);
                    } else {
                        let v: Vec<_> = output.drain().collect();
                        let v = v
                            .get(0)
                            .cloned()
                            .map(|(_, val)| val.clone())
                            .unwrap_or(Value::Null);
                        return v;
                    }
                },
            },
            // OMGenOne {
            //     origin: self.clone(),
            //     from_many: if let Operation::SelectMany = op {
            //         true
            //     } else {
            //         false
            //     },
            //     output: Default::default(),
            // },
        ))
    }
    fn init_insert(
        self: Arc<Self>,
        to: &str,
        input: Value,
    ) -> Result<Box<dyn OnInsertRelation + Send + Sync>, String>
    {
        if to != self.to {
            return Err(format!(
                "relation for \"{}\" not found",
                to
            ));
        }
        let input =
            match serde_json::from_value::<InsertIn>(input) {
                Ok(input) => input,
                Err(err) => return Err(err.to_string()),
            };

        Ok(Box::new(OMGenOneInsert {
            origin: self.clone(),
            input,
        }))
    }

    fn init_update(
        self: Arc<Self>,
        to: &str,
        value: Value,
    ) -> Result<Box<dyn OnUpdateRelation + Send + Sync>, String>
    {
        if to != self.to {
            return Err(format!(
                "relation for \"{}\" not found",
                to
            ));
        }

        let input =
            match serde_json::from_value::<UpdateIn>(value) {
                Ok(input) => input,
                Err(err) => return Err(err.to_string()),
            };

        Ok(Box::new(OMGenOneUpdate {
            origin: self.clone(),
            input,
        }))
    }
}
