use std::marker::PhantomData;

use queries_for_sqlx::{
    impl_into_mut_arguments_prelude::Type, BindItem, Constraint,
    Query, SchemaColumn,
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
        "PRIMARY KEY"
    }
}

pub struct ColumnTypeCheckIfNull<T>(PhantomData<T>);

impl<S, T> SchemaColumn<S> for ColumnTypeCheckIfNull<T>
where
    S: Database,
    T: sqlx::Type<S> + IsNull,
{
    fn display(&self) -> String {
        use sqlx::TypeInfo;
        let ty = T::type_info();
        let ty = ty.name().to_string();

        format!(
            "{}{}",
            ty,
            if T::is_null() { "" } else { " NOT NULL" }
        )
    }
}

impl<S, Q, T> BindItem<S, Q> for ColumnTypeCheckIfNull<T>
where
    S: Database,
    Q: Query,
    T: Type<S> + IsNull + 'static,
{
    fn bind_item(
        self,
        ctx: &mut <Q as Query>::Context1,
    ) -> impl FnOnce(&mut <Q as Query>::Context2) -> String + 'static
    {
        move |_| self.display()
    }
}

pub fn col_type_check_if_null<T>() -> ColumnTypeCheckIfNull<T> {
    ColumnTypeCheckIfNull(PhantomData)
}

pub struct DefaultPrimaryKey;

impl<S: SqlxQuery> SchemaColumn<S> for DefaultPrimaryKey {
    fn display(&self) -> String {
        S::default_primary_key().to_string()
    }
}

impl<S, Q> BindItem<S, Q> for DefaultPrimaryKey
where
    S: Database + SqlxQuery,
    Q: Query,
{
    fn bind_item(
        self,
        ctx: &mut <Q as Query>::Context1,
    ) -> impl FnOnce(&mut <Q as Query>::Context2) -> String + 'static
    {
        move |_| <Self as SchemaColumn<S>>::display(&self)
    }
}

pub struct PK<S>(PhantomData<S>);

pub fn primary_key<S>() -> PK<S> {
    PK(PhantomData)
}

impl<S> SchemaColumn<S> for PK<S>
where
    S: SqlxQuery,
    S: Database,
{
    fn display(&self) -> String {
        use sqlx::TypeInfo;
        let ty = S::KeyType::type_info().to_string();
        format!("{} {}", ty, S::default_primary_key())
    }
}

impl<S, Q> BindItem<S, Q> for PK<S>
where
    S: Database,
    S: SqlxQuery,
    Q: Query,
{
    fn bind_item(
        self,
        ctx: &mut <Q as Query>::Context1,
    ) -> impl FnOnce(&mut <Q as Query>::Context2) -> String + 'static
    {
        move |_| <Self as SchemaColumn<S>>::display(&self)
    }
}
