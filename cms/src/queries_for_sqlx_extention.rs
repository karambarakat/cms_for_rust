use std::marker::PhantomData;

use queries_for_sqlx::{
    impl_into_mut_arguments_prelude::Type, Constraint, Query,
    SchemaColumn,
};
use sqlx::Database;

pub trait IsNull {
    fn is_null() -> bool;
}

mod impl_is_null_no_spectialization {
    use super::IsNull;

    impl<T> IsNull for Option<T> {
        fn is_null() -> bool {
            true
        }
    }

    macro_rules! impl_no_gens {
            ($($ident:ident)*) => {
                $(impl IsNull for $ident {
                    fn is_null() -> bool {
                        false
                    }
                })*
            };
        }

    impl_no_gens!(i32 i64 bool char String);
}

pub trait SqlxQuery: Database {
    type KeyType: Type<Self> + IsNull + Send + Sync + 'static;
    fn default_primary_key() -> &'static str;
}

impl SqlxQuery for sqlx::Sqlite {
    type KeyType = i64;
    fn default_primary_key() -> &'static str {
        "PRIMARY KEY AUTOINCREMENT"
    }
}

impl SqlxQuery for sqlx::Postgres {
    type KeyType = i64;
    fn default_primary_key() -> &'static str {
        "PRIMARY KEY AUTOINCREMENT"
    }
}

pub struct ColumnTypeCheckIfNull<T>(PhantomData<T>);

impl<S, T, Q> SchemaColumn<S, Q> for ColumnTypeCheckIfNull<T>
where
    S: Database,
    Q: Query<S>,
    T: sqlx::Type<S> + IsNull,
{
    fn column(
        self,
        _: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String {
        use sqlx::TypeInfo;
        let ty = T::type_info();
        let ty = ty.name().to_string();
        move |_| {
            format!(
                "{}{}",
                ty,
                if T::is_null() { "" } else { " NOT NULL" }
            )
        }
    }
}

pub fn col_type_check_if_null<T>() -> ColumnTypeCheckIfNull<T> {
    ColumnTypeCheckIfNull(PhantomData)
}

pub struct DefaultPrimaryKey;

impl<S: SqlxQuery, Q> SchemaColumn<S, Q> for DefaultPrimaryKey
where
    Q: Query<S>,
{
    fn column(
        self,
        _: &mut <Q as Query<S>>::Context1,
    ) -> impl FnOnce(&mut <Q as Query<S>>::Context2) -> String {

        |_| S::default_primary_key().to_string()
    }
}

pub fn primary_key() -> DefaultPrimaryKey {
    DefaultPrimaryKey
}
