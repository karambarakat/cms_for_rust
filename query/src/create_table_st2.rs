use std::marker::PhantomData;

use sqlx::{Database, Type};

use crate::{
    sql_part::{
        ColumnToSqlPart, ConstraintToSqlPart, ToSqlPart,
    },
    Query, Statement,
};

pub struct CreateTableSt<S, Q: Query<S>> {
    pub(crate) header: String,
    pub(crate) ident: (Option<String>, String),
    pub(crate) columns: Vec<(String, Q::SqlPart)>,
    pub(crate) constraints: Vec<Q::SqlPart>,
    pub(crate) verbatim: Vec<String>,
    pub(crate) ctx: Q::Context1,
    pub(crate) _sqlx: PhantomData<S>,
}

impl<S, Q> Statement<S, Q> for CreateTableSt<S, Q>
where
    Q: Query<S>,
{
    type Init = &'static str;
    fn init(header: &'static str) -> Self {
        Self {
            header: header.to_string(),
            ident: Default::default(),
            columns: Default::default(),
            constraints: Default::default(),
            verbatim: Default::default(),
            ctx: Default::default(),
            _sqlx: PhantomData,
        }
    }
    fn _build(self) -> (String, Q::Output) {
        Q::build_query(self.ctx, |ctx| {
            let mut str = String::from(&self.header);

            str.push(' ');

            if let Some(schema) = self.ident.0 {
                str.push_str(&schema);
            }

            str.push_str(&self.ident.1);

            str.push_str(" (");

            let mut clauses = Vec::new();
            for col in self.columns {
                let item = Q::build_sql_part_back(ctx, col.1);
                clauses.push(format!("{} {}", col.0, item));
            }
            for constraint in self.constraints {
                let item =
                    Q::build_sql_part_back(ctx, constraint);
                clauses.push(item);
            }

            for verbatim in self.verbatim {
                clauses.push(verbatim);
            }
            str.push_str(&clauses.join(", "));
            str.push_str(");");

            str
        })
    }
}

impl<S, Q> CreateTableSt<S, Q>
where
    Q: Query<S>,
    S: Database,
{
    pub fn verbatim(&mut self, verbatim: &str) {
        self.verbatim.push(verbatim.to_string());
    }
    pub fn column_no_constraint<Ty>(&mut self, name: &str)
    where
        Ty: Type<S>,
    {
        self.columns.push((
            name.to_string(),
            todo!(),
            // ColumnToSqlPart(
            //
            //     , PhantomData::<Q>)
            //     .to_sql_part(&mut self.ctx),
        ));
    }
    pub fn column<C>(&mut self, name: &str, constraint: C)
    where
        C: SchemaColumn<S>,
        ColumnToSqlPart<C, Q>: ToSqlPart<Q, S>,
    {
        self.columns.push((
            name.to_string(),
            ColumnToSqlPart(constraint, PhantomData::<Q>)
                .to_sql_part(&mut self.ctx),
        ));
    }
    pub fn constraint<C>(&mut self, constraint: C)
    where
        C: Constraint<S>,
        ConstraintToSqlPart<C, Q>: ToSqlPart<Q, S>,
    {
        self.constraints.push(
            constraint.reflixive().to_sql_part(&mut self.ctx),
        )
    }
}

use new_column_trait::*;
pub(crate) mod new_column_trait {
    use crate::Query;

    pub trait SchemaColumn<S>: Sized {
        fn column<Q>(
            self,
            ctx: &mut Q::Context1,
        ) -> impl FnOnce(&mut Q::Context2) -> String
        where
            Q: Query<S>;
    }
}

use new_constraint_trait::*;
pub(crate) mod new_constraint_trait {
    use std::marker::PhantomData;

    use crate::{sql_part::ConstraintToSqlPart, Query};

    pub trait Constraint<S>: Sized {
        fn reflixive<Q>(self) -> ConstraintToSqlPart<Self, Q> {
            ConstraintToSqlPart(self, PhantomData)
        }
        fn constraint<Q>(
            self,
            ctx: &mut Q::Context1,
        ) -> impl FnOnce(&mut Q::Context2) -> String
        where
            Q: Query<S>;
    }

    #[derive(Clone)]
    pub struct ForiegnKey {
        pub not_null: bool,
        pub column: Option<&'static str>,
        pub refer_table: Option<&'static str>,
        pub refer_column: Option<&'static str>,
    }

    impl ForiegnKey {
        pub fn build() -> Self {
            Self {
                not_null: false,
                column: None,
                refer_table: None,
                refer_column: None,
            }
        }
        #[track_caller]
        pub fn finish(&mut self) -> Self {
            if self.column.is_none() {
                panic!("column is required");
            }
            if self.refer_table.is_none() {
                panic!("refer_table is required");
            }
            if self.refer_column.is_none() {
                panic!("refer_column is required");
            }
            self.to_owned()
        }
        pub fn not_null(&mut self) -> &mut Self {
            self.not_null = true;
            self
        }
        pub fn column(
            &mut self,
            column: &'static str,
        ) -> &mut Self {
            self.column = Some(column);
            self
        }
        pub fn refer_table(
            &mut self,
            refer_table: &'static str,
        ) -> &mut Self {
            self.refer_table = Some(refer_table);
            self
        }
        pub fn refer_column(
            &mut self,
            refer_column: &'static str,
        ) -> &mut Self {
            self.refer_column = Some(refer_column);
            self
        }
    }

    impl<S> Constraint<S> for ForiegnKey {
        fn constraint<Q>(
            self,
            _: &mut Q::Context1,
        ) -> impl FnOnce(&mut Q::Context2) -> String
        where
            Q: Query<S>,
        {
            move |_| {
                format!(
                "FOREIGN KEY ({}) REFERENCES {}({})",
                self.column.expect("should have set a column on foreign_key"), 
                self.refer_table.expect("should have set a refer_table on foreign_key"), 
                self.refer_column.expect("should have set a refer_column on foreign_key")
            )
            }
        }
    }
}

#[cfg(test)]
mod test_create_table_st {
    use sqlx::Sqlite;

    use crate::debug_query::DebugQuery;

    use super::*;

    fn test_default<'q>() -> CreateTableSt<Sqlite, DebugQuery> {
        CreateTableSt {
            header: "CREATE TABLE".to_string(),
            ident: (None, "users".to_string()),
            columns: vec![],
            constraints: vec![],
            ctx: Default::default(),
            verbatim: Default::default(),
            _sqlx: PhantomData,
        }
    }

    #[test]
    fn test_foreign_key() {
        let mut table = test_default();

        table.constraint(
            ForiegnKey::build()
                .column("id")
                .refer_table("users")
                .refer_column("id")
                .finish(),
        );

        let (str, _) = table._build();

        assert_eq!(
            str,
            "CREATE TABLE users (FOREIGN KEY (id) REFERENCES users(id));"
        );
    }
}
