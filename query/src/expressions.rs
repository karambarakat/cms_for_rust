use std::marker::PhantomData;

use aliasing::HasLocal;
use sqlx::Database;

use crate::{
    Accept, Constraint, Query, SchemaColumn, SelectItem,
    WhereItem,
};

pub struct Column {
    pub(crate) name: &'static str,
}

impl HasLocal for Column {
    fn local_col(&self) -> &'static str {
        self.name
    }
}

impl<S> SelectItem<S> for Column {
    fn select_item(self) -> String {
        self.name.to_string()
    }
}

pub struct ColumnEq<Col, T> {
    pub(crate) column: Col,
    pub(crate) value: T,
}

// todo: do I have to give generic impl for every 'impl Select'
impl ForeignColumn {
    pub fn eq<T>(self, value: T) -> ColumnEq<Self, T> {
        ColumnEq {
            column: self,
            value,
        }
    }
}
impl Column {
    pub fn eq<T>(self, value: T) -> ColumnEq<Self, T> {
        ColumnEq {
            column: self,
            value,
        }
    }
}

impl<S, Col, Q: Query<S>, T> WhereItem<S, Q> for ColumnEq<Col, T>
where
    Col: SelectItem<S>,
    Q: Accept<T, S>,
{
    fn where_item(
        self,
        ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String {
        let column = self.column;
        let value = Q::accept(self.value, ctx);
        move |ctx| {
            format!("{} = {}", column.select_item(), value(ctx))
        }
    }
}

pub struct ForeignTable {
    pub(crate) name: &'static str,
}

pub struct ForeignColumn {
    pub(crate) table: &'static str,
    pub(crate) column: &'static str,
}

impl HasLocal for ForeignColumn {
    fn local_col(&self) -> &'static str {
        self.column
    }
}

impl<S> SelectItem<S> for ForeignColumn {
    fn select_item(self) -> String {
        format!("{}.{}", self.table, self.column)
    }
}

impl ForeignTable {
    pub fn col(self, column: &'static str) -> ForeignColumn {
        ForeignColumn {
            table: self.name,
            column,
        }
    }
}

pub mod aliasing {

    use case::CaseExt;

    use crate::SelectItem;

    pub struct Alias<E, S> {
        pub(crate) expr: E,
        pub(crate) alias: String,
        _pd: std::marker::PhantomData<S>,
    }

    impl<S, I> SelectItem<S> for Alias<I, S>
    where
        I: SelectItem<S>,
    {
        fn select_item(self) -> String {
            format!(
                "{} AS {}",
                self.expr.select_item(),
                self.alias
            )
        }
    }

    pub struct PrefixAlias<E, S> {
        pub(crate) expr: E,
        pub(crate) prefix: &'static str,
        _pd: std::marker::PhantomData<S>,
    }

    impl<S, I> SelectItem<S> for PrefixAlias<I, S>
    where
        I: SelectItem<S> + HasLocal,
    {
        fn select_item(self) -> String {
            let local = self.expr.local_col();
            format!(
                "{} AS {}_{}",
                self.expr.select_item(),
                self.prefix.to_snake(),
                local
            )
        }
    }

    pub trait HasLocal {
        fn local_col(&self) -> &'static str;
    }

    pub trait SelectHelpers<S>: SelectItem<S> + Sized {
        fn alias2(self, alias: String) -> Alias<Self, S>
        where
            Self: Sized,
        {
            Alias {
                expr: self,
                alias,
                _pd: std::marker::PhantomData,
            }
        }
        fn prefix_alias(
            self,
            prefix: &'static str,
        ) -> PrefixAlias<Self, S>
        where
            Self: Sized,
            Self: HasLocal,
        {
            PrefixAlias {
                expr: self,
                prefix,
                _pd: std::marker::PhantomData,
            }
        }
        fn alias(self, alias: &'static str) -> Alias<Self, S>
        where
            Self: Sized,
        {
            Alias {
                expr: self,
                alias: alias.to_string(),
                _pd: std::marker::PhantomData,
            }
        }
    }

    impl<S, E> SelectHelpers<S> for E where E: SelectItem<S> {}
}

pub struct IsNullWhereItem<T, S>(T, PhantomData<S>);

impl<S, Q: Query<S>, T: SelectItem<S>> WhereItem<S, Q>
    for IsNullWhereItem<T, S>
{
    fn where_item(
        self,
        _ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String {
        |_ctx2| format!("{} IS NULL", self.0.select_item())
    }
}
pub struct IsNotNullWhereItem<T, S>(T, PhantomData<S>);

impl<S, Q: Query<S>, T: SelectItem<S>> WhereItem<S, Q>
    for IsNotNullWhereItem<T, S>
{
    fn where_item(
        self,
        _ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String {
        |_ctx2| format!("{} IS NOT NULL", self.0.select_item())
    }
}

pub trait SelectHelpers2<S>: SelectItem<S> + Sized {
    fn is_not_null(self) -> IsNotNullWhereItem<Self, S> {
        IsNotNullWhereItem(self, PhantomData)
    }
    fn is_null(self) -> IsNullWhereItem<Self, S> {
        IsNullWhereItem(self, PhantomData)
    }
}

impl<S, E> SelectHelpers2<S> for E where E: SelectItem<S> {}

pub struct VerbatimNoSanitize(String);

impl<S> SelectItem<S> for VerbatimNoSanitize {
    fn select_item(self) -> String {
        self.0
    }
}

impl<S, Q: Query<S>> WhereItem<S, Q> for VerbatimNoSanitize {
    fn where_item(
        self,
        _ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String {
        move |_ctx| self.0
    }
}

pub struct AllColumns;

impl<S> SelectItem<S> for AllColumns {
    fn select_item(self) -> String {
        "*".to_string()
    }
}

#[derive(Clone)]
pub struct ForiegnKey {
    not_null: bool,
    column: Option<&'static str>,
    refer_table: Option<&'static str>,
    refer_column: Option<&'static str>,
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
    pub fn column(&mut self, column: &'static str) -> &mut Self {
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

impl<S, Q: Query<S>> Constraint<S, Q> for ForiegnKey {
    fn constraint(
        self,
        _: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String {
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

pub struct ColumnType<T>(PhantomData<T>);

impl<Q: Query<S>, S, T> SchemaColumn<S, Q> for ColumnType<T>
where
    S: Database,
    T: sqlx::Type<S>,
{
    fn column(
        self,
        _: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String {
        use sqlx::TypeInfo;
        let ty = T::type_info();
        let ty = ty.name().to_string();
        move |_| format!("{}", ty,)
    }
}

pub struct DefaultConstraint<Closure, T>(
    Closure,
    PhantomData<T>,
);

// impl<S, Ty, T> SchemaColumn<S> for DefaultConstraint<Ty, T>
// where
//     Q: Accept<T, S>,
// {
//     fn column<Q>(
//         self,
//         ctx: &mut Q::Context1,
//     ) -> impl FnOnce(&mut Q::Context2) -> String
//     where
//         Q: Query<S>,
//     {
//         let save = Q::accept(self.0, ctx);
//         |ctxr| format!("DEFAULT {}", save(ctx))
//     }
// }

pub struct NotNull;

impl<S, Q: Query<S>> SchemaColumn<S, Q> for NotNull
where
    S: Database,
{
    fn column(
        self,
        _: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String {
        move |_| "NOT NULL".to_string()
    }
}

pub mod exports {
    pub use super::aliasing::SelectHelpers;
    pub use super::SelectHelpers2;

    use super::*;

    pub fn col(name: &'static str) -> Column {
        Column { name }
    }

    pub fn ft(name: &'static str) -> ForeignTable {
        ForeignTable { name }
    }

    pub fn all_columns() -> AllColumns {
        AllColumns
    }

    #[allow(non_snake_case)]
    pub fn verbatim__warning__does_not_sanitize(
        s: String,
    ) -> VerbatimNoSanitize {
        VerbatimNoSanitize(s)
    }

    pub fn foreign_key() -> ForiegnKey {
        ForiegnKey::build()
    }

    pub fn col_type<T>() -> ColumnType<T> {
        ColumnType(PhantomData)
    }
}
