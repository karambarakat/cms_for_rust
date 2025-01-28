use std::{fmt::Display, marker::PhantomData};

use sqlx::{database::HasArguments, Database};

use crate::{
    execute_no_cache::ExecuteNoCacheUsingSelectTrait,
    ident_safety::PanicOnUnsafe, AcceptTableIdent, BindItem,
    Constraint, IdentSafety, Query, QueryHandlers, SchemaColumn,
    Statement,
};

#[derive(Debug)]
pub struct CreateTableSt<S, Q: Query, I: IdentSafety> {
    pub(crate) header: String,
    pub(crate) ident: (Option<String>, I::Table),
    pub(crate) columns: Vec<(String, Q::SqlPart)>,
    pub(crate) constraints: Vec<Q::SqlPart>,
    pub(crate) verbatim: Vec<String>,
    pub(crate) ctx: Q::Context1,
    pub(crate) _sqlx: PhantomData<S>,
}

pub enum CreateTableHeader {
    Create,
    CreateTemp,
    CreateTempIfNotExists,
    IfNotExists,
}

impl Display for CreateTableHeader {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            CreateTableHeader::Create => write!(f, "CREATE"),
            CreateTableHeader::CreateTemp => {
                write!(f, "CREATE TEMP")
            }
            CreateTableHeader::CreateTempIfNotExists => {
                write!(f, "CREATE TEMP IF NOT EXISTS")
            }
            CreateTableHeader::IfNotExists => {
                write!(f, "CREATE TABLE IF NOT EXISTS")
            }
        }
    }
}

impl<S, Q: Query, I: IdentSafety> ExecuteNoCacheUsingSelectTrait
    for CreateTableSt<S, Q, I>
{
}

impl<S, Q, I> CreateTableSt<S, Q, I>
where
    Q: Query,
    I: IdentSafety,
{
    pub fn init<C>(header: (CreateTableHeader, C)) -> Self
    where
        I: AcceptTableIdent<C>,
    {
        Self {
            header: header.0.to_string(),
            ident: (None, I::into_table(header.1)),
            columns: Default::default(),
            constraints: Default::default(),
            verbatim: Default::default(),
            ctx: Default::default(),
            _sqlx: PhantomData,
        }
    }
}

impl<S, Q, I> Statement<S, Q> for CreateTableSt<S, Q, I>
where
    Q: Query,
    I: IdentSafety,
{
    fn deref_ctx(&self) -> &Q::Context1 {
        &self.ctx
    }
    fn deref_mut_ctx(&mut self) -> &mut Q::Context1 {
        &mut self.ctx
    }
    #[track_caller]
    fn _build(self) -> (String, Q::Output) {
        Q::build_query(self.ctx, |ctx| {
            let mut str = String::from(&self.header);

            str.push(' ');

            if let Some(schema) = self.ident.0 {
                str.push_str(&schema);
            }

            str.push_str(self.ident.1.as_ref());

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
            if clauses.is_empty() {
                panic!("columns is empty");
            }
            str.push_str(&clauses.join(", "));
            str.push_str(");");

            str
        })
    }
}

impl<S, Q, I> CreateTableSt<S, Q, I>
where
    I: IdentSafety,
    Q: Query,
    S: Database,
{
    pub fn verbatim(&mut self, verbatim: &str) {
        self.verbatim.push(verbatim.to_string());
    }
    pub fn column<C>(& mut self, name: &str, constraint: C)
    where
        C: SchemaColumn<S> + 'static,
        C: BindItem<S, Q, I>,
        Q: QueryHandlers<S>,
    {
        let item =
            Q::handle_bind_item(constraint, &mut self.ctx);
        self.columns.push((name.to_string(), item));
    }
    pub fn constraint<C>(& mut self, constraint: C)
    where
        C: Constraint + 'static,
        C: BindItem<S, Q, I>,
        Q: QueryHandlers<S>,
    {
        let item =
            Q::handle_bind_item(constraint, &mut self.ctx);
        self.constraints.push(item)
    }
}

// #[cfg(todo)]
// mod create_table_st {
//     use sqlx::{Pool, Sqlite};
//
//     use crate::{
//         expressions::{
//             exports::{col_type, foreign_key},
//             NotNull,
//         },
//         SupportNamedBind,
//     };
//
//     use super::*;
//
//     fn test_default<'q>() -> CreateTableSt<Sqlite, DebugQuery> {
//         CreateTableSt {
//             header: "CREATE TABLE".to_string(),
//             ident: (None, "users".to_string()),
//             columns: vec![],
//             constraints: vec![],
//             ctx: Default::default(),
//             verbatim: Default::default(),
//             _sqlx: PhantomData,
//         }
//     }
//
//     trait QueryIsDebug<S>: Sized {
//         fn query_is_debug(self) -> Self {
//             self
//         }
//     }
//     impl<S, T> QueryIsDebug<S> for T
//     where
//         S: Database + SupportNamedBind,
//         T: crate::Statement<S, DebugQuery> + Sized,
//     {
//     }
//
//     use crate::execute_no_cache::ExecuteNoCache;
//
//     #[tokio::test]
//     async fn create_main() {
//         let pool = Pool::<Sqlite>::connect("sqlite::memory:")
//             .await
//             .unwrap();
//         let mut st = CreateTableSt::init((
//             CreateTableHeader::IfNotExists,
//             "Todo",
//         ))
//         .query_is_debug();
//
//         st.column("id", (col_type::<i64>(), NotNull));
//
//         assert_eq!(
//             st.build_statement(),
//             "CREATE TABLE IF NOT EXISTS Todo (id INTEGER NOT NULL);"
//         );
//
//         st.execute(&pool).await.unwrap();
//     }
//
//     #[test]
//     fn test_foreign_key() {
//         let mut table = test_default();
//
//         table.constraint(
//             foreign_key()
//                 .column("id")
//                 .refer_table("users")
//                 .refer_column("id")
//                 .finish(),
//         );
//
//         let (str, _) = table._build();
//
//         assert_eq!(
//             str,
//             "CREATE TABLE users (FOREIGN KEY (id) REFERENCES users(id));"
//         );
//     }
// }
