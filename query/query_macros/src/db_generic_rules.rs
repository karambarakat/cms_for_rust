#![allow(unreachable_patterns)]

support!(
    Database,
    "$$DB: sqlx::Database",
    [Sqlite, Postgres, MySQL, MariaDB, MSSQL, Oracle]
);

support!(
    Executer,
    "for<'db> 
        &'db mut <$$DB as sqlx::Database>::Connection: 
        sqlx::Executor<'db, Database = $$DB>",
    [Sqlite, Postgres, MySQL, MariaDB, MSSQL, Oracle]
);

support!(
    HasArguments,
    "for<'db> 
        <$$DB as sqlx::database::HasArguments<'db>>::Arguments: 
        sqlx::IntoArguments<'db, $$DB>",
    [Sqlite, Postgres, MySQL, MariaDB, MSSQL, Oracle]
);

support_each_type!(
    Decode,
    "for<'db> {}:
        sqlx::Decode<'db, $$DB> + sqlx::Type<$$DB>",
    [
        Sqlite: ["i8", "i32", "String", "&'db str"],
        Postgres: ["i64", "i32", "String", "&'db str"],
        MySQL: ["i64", "i32", "String"],
        MariaDB: ["i64", "i32", "String"],
        MSSQL: ["i64", "i32", "String"],
        Oracle: ["i64", "i32", "String"]
    ]
);

support_each_type!(
    DecodeOption,
    "for<'db> Option<{}>:
        sqlx::Decode<'db, $$DB> + sqlx::Type<$$DB>",
    [
        Sqlite: ["i64", "i8", "i32", "String", "&'db str"],
        Postgres: ["i64", "i32", "String", "&'db str"],
        MySQL: ["i64", "i32", "String"],
        MariaDB: ["i64", "i32", "String"],
        MSSQL: ["i64", "i32", "String"],
        Oracle: ["i64", "i32", "String"]
    ]
);

support_each_type!(
    EncodeOption,
    "for<'db> Option<{}>:
        sqlx::Encode<'db, $$DB> + sqlx::Type<$$DB>",
    [
        Sqlite: ["i64", "i32", "String"],
        Postgres: ["i64", "i32", "String"],
        MySQL: ["i64", "i32", "String"]
    ]
);

support_each_type!(
    Encode,
    "for<'db> {}:
        sqlx::Encode<'db, $$DB> + sqlx::Type<$$DB>",
    [
        Sqlite: ["i64", "i32", "String"],
        Postgres: ["i64", "i32", "String"],
        MySQL: ["i64", "i32", "String"],
        MariaDB: ["i64", "i32", "String"],
        MSSQL: ["i64", "i32", "String"],
        Oracle: ["i64", "i32", "String"]
    ]
);

support!(
    ColumnIndexRef,
    "for<'db> &'db str: 
        sqlx::ColumnIndex<<$$DB as sqlx::Database>::Row>",
    [Sqlite, Postgres, MySQL, MariaDB, MSSQL, Oracle]
);

support_each_type!(
    ColumnIndex,
    "{}:
        sqlx::ColumnIndex<<$$DB as sqlx::Database>::Row>",
    [
        Sqlite: ["usize"],
        Postgres: ["String", "usize"],
        MySQL: ["String", "usize"],
        MariaDB: ["String", "usize"],
        MSSQL: ["String", "usize"],
        Oracle: ["String", "usize"]
    ]
);

support!(
    BindNamed,
    "$$DB: ::queries_for_sqlx::SupportNamedBind",
    [Sqlite, Postgres]
);



pub(crate) fn rules() -> Vec<Box<dyn WherePredicate>> {
    vec![
        Database.to_dyn(),
        Executer.to_dyn(),
        HasArguments.to_dyn(),
        Encode.to_dyn(),
        Decode.to_dyn(),
        EncodeOption.to_dyn(),
        DecodeOption.to_dyn(),
        ColumnIndexRef.to_dyn(),
        ColumnIndex.to_dyn(),
        BindNamed.to_dyn(),
    ]
}

macro_rules! convert {
    ($ident:ident) => {
        impl From<$ident> for Box<dyn $crate::db_generic::WherePredicate> {
            fn from(_: $ident) -> Self {
                Box::new($ident)
            }
        }
        impl $ident {
            pub fn to_dyn(self) -> Box<dyn $crate::db_generic::WherePredicate> {
                self.into()
            }
        }
    };
}

pub(crate) use convert;

macro_rules! support {
    ($struct_name:ident, $predicate:expr, [$($db:ident),*]) => {
        pub(crate) struct $struct_name;
        impl $crate::db_generic::WherePredicate for $struct_name {
            fn vec(&self, dbs: $crate::db_generic::DBs) -> Vec<String> {
                match dbs {
                    $($crate::db_generic::DBs::$db => vec![$predicate.to_owned()],)*
                    _ => vec![],
                }
            }
        }

        $crate::db_generic_rules::convert!($struct_name);
    };
}

pub(crate) use support;

macro_rules! support_each_type {
    ($struct_name:ident, $predicate:expr, [$($db:ident : [$($ty:literal),*]),*]) => {
        pub(crate) struct $struct_name;
        impl $crate::db_generic::WherePredicate for $struct_name {
            fn vec(&self, dbs: $crate::db_generic::DBs) -> Vec<String> {
                match dbs {
                    $($crate::db_generic::DBs::$db => vec![$($ty.to_owned()),*],)*
                    _ => vec![],
                }
                .into_iter()
                .map(|ty| format!($predicate, ty))
                .collect()
            }
        }

        $crate::db_generic_rules::convert!($struct_name);
    };
}

pub(crate) use support_each_type;

use crate::db_generic::*;
