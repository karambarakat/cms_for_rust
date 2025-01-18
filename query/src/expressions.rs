use std::marker::PhantomData;

use aliasing::HasLocal;
use sqlx::Database;

use crate::{
    ident_safety::IdentSafety, Accept, BindItem, Constraint,
    Query, SchemaColumn, SelectItem,
};

pub struct Column<I: IdentSafety> {
    pub(crate) name: I::Column,
}

impl<I: IdentSafety> HasLocal<I> for Column<I> {
    fn local_col(&self) -> &I::Column {
        &self.name
    }
}

impl<S, I: IdentSafety> SelectItem<S, I> for Column<I> {
    fn select_item(self) -> String {
        self.name.as_ref().to_string()
    }
}

pub struct ColumnEq<Col, T, I> {
    pub(crate) column: Col,
    pub(crate) value: T,
    pub(crate) _pd: PhantomData<I>,
}

// todo: do I have to give generic impl for every 'impl Select'
impl<I: IdentSafety> ForeignColumn<I> {
    pub fn eq<T>(self, value: T) -> ColumnEq<Self, T, I> {
        ColumnEq {
            column: self,
            value,
            _pd: PhantomData,
        }
    }
}

impl<I: IdentSafety> Column<I> {
    pub fn eq<T>(self, value: T) -> ColumnEq<Self, T, I> {
        ColumnEq {
            column: self,
            value,
            _pd: PhantomData,
        }
    }
}

impl<S, Col, Q: Query, T> BindItem<S, Q>
    for ColumnEq<Col, T, Q::IdentSafety>
where
    Col: SelectItem<S, Q::IdentSafety> + 'static,
    Q: Accept<T, S> + 'static,
{
    fn bind_item(
        self,
        ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String + 'static {
        let column = self.column;
        let value = Q::accept(self.value, ctx);
        move |ctx| {
            format!("{} = {}", column.select_item(), value(ctx))
        }
    }
}

pub struct ForeignTable<I: IdentSafety> {
    pub(crate) name: I::Table,
}

pub struct ForeignColumn<I: IdentSafety> {
    pub(crate) table: I::Table,
    pub(crate) column: I::Column,
}

impl<I: IdentSafety> HasLocal<I> for ForeignColumn<I> {
    fn local_col(&self) -> &I::Column {
        &self.column
    }
}

impl<S, I: IdentSafety> SelectItem<S, I> for ForeignColumn<I> {
    fn select_item(self) -> String {
        format!(
            "{}.{}",
            self.table.as_ref(),
            self.column.as_ref()
        )
    }
}

impl<I: IdentSafety> ForeignTable<I> {
    pub fn col(self, column: I::Column) -> ForeignColumn<I> {
        ForeignColumn {
            table: self.name,
            column,
        }
    }
}

pub mod aliasing {

    use case::CaseExt;

    use crate::{ident_safety::IdentSafety, SelectItem};

    pub struct Alias<E, S> {
        pub(crate) expr: E,
        pub(crate) alias: String,
        _pd: std::marker::PhantomData<S>,
    }

    impl<S, E, I> SelectItem<S, I> for Alias<E, S>
    where
        E: SelectItem<S, I>,
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

    impl<S, E, I> SelectItem<S, I> for PrefixAlias<E, S>
    where
        I: IdentSafety,
        I::Column: Clone,
        E: SelectItem<S, I> + HasLocal<I>,
    {
        fn select_item(self) -> String {
            let local = self.expr.local_col().clone();
            format!(
                "{} AS {}_{}",
                self.expr.select_item(),
                self.prefix.to_snake(),
                local.as_ref()
            )
        }
    }

    pub trait HasLocal<I: IdentSafety> {
        fn local_col(&self) -> &I::Column;
    }

    pub trait SelectHelpers<S, I>:
        SelectItem<S, I> + Sized
    where
        I: IdentSafety,
    {
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
            Self: HasLocal<I>,
        {
            PrefixAlias {
                expr: self,
                prefix,
                _pd: std::marker::PhantomData,
            }
        }
        fn alias(self, alias: String) -> Alias<Self, S>
        where
            Self: Sized,
        {
            Alias {
                expr: self,
                alias,
                _pd: std::marker::PhantomData,
            }
        }
    }

    impl<S, E, I: IdentSafety> SelectHelpers<S, I> for E where
        E: SelectItem<S, I>
    {
    }
}

pub struct IsNullWhereItem<T, S>(T, PhantomData<S>);

impl<S, Q, T> BindItem<S, Q> for IsNullWhereItem<T, S>
where
    Q: Query,
    T: SelectItem<S, Q::IdentSafety> + 'static,
{
    fn bind_item(
        self,
        _ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String + 'static {
        |_ctx2| format!("{} IS NULL", self.0.select_item())
    }
}
pub struct IsNotNullWhereItem<T, S>(T, PhantomData<S>);

impl<S, Q: Query, I, T: SelectItem<S, I>> BindItem<S, Q>
    for IsNotNullWhereItem<T, S>
{
    fn bind_item(
        self,
        _ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String {
        |_ctx2| format!("{} IS NOT NULL", self.0.select_item())
    }
}

pub trait SelectHelpers2<S, I>:
    SelectItem<S, I> + Sized
{
    fn is_not_null(self) -> IsNotNullWhereItem<Self, S> {
        IsNotNullWhereItem(self, PhantomData)
    }
    fn is_null(self) -> IsNullWhereItem<Self, S> {
        IsNullWhereItem(self, PhantomData)
    }
}

impl<S, E, I> SelectHelpers2<S, I> for E where E: SelectItem<S, I>
{}

pub struct VerbatimNoSanitize(String);

impl<S, I> SelectItem<S, I> for VerbatimNoSanitize {
    fn select_item(self) -> String {
        self.0
    }
}

impl<S, Q: Query, I> BindItem<S, Q> for VerbatimNoSanitize {
    fn bind_item(
        self,
        _ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String {
        move |_ctx| self.0
    }
}

pub struct AllColumns;

impl<S, I> SelectItem<S, I> for AllColumns {
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
    on_delete_clause: Option<&'static str>,
}

impl ForiegnKey {
    pub fn build() -> Self {
        Self {
            not_null: false,
            column: None,
            refer_table: None,
            refer_column: None,
            on_delete_clause: None,
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
    pub fn on_delete_set_null(&mut self) -> &mut Self {
        self.on_delete_clause = Some("ON DELETE SET NULL");
        self
    }
    pub fn on_delete_cascade(&mut self) -> &mut Self {
        self.on_delete_clause = Some("ON DELETE CASCADE");
        self
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

impl Constraint for ForiegnKey {}
impl<S, Q: Query> BindItem<S, Q> for ForiegnKey {
    fn bind_item(
        self,
        ctx: &mut <Q as Query>::Context1,
    ) -> impl FnOnce(&mut <Q as Query>::Context2) -> String + 'static
    {
        move |ctx2| {
            format!(
                "FOREIGN KEY ({}) REFERENCES {}({}){}",
                self.column.expect("should have set a column on foreign_key"), 
                self.refer_table.expect("should have set a refer_table on foreign_key"), 
                self.refer_column.expect("should have set a refer_column on foreign_key"),
                match self.on_delete_clause {
                    Some(s) => format!(" {}", s),
                    None => "".to_string(),
                }

            )
        }
    }
}

pub struct ColumnType<T>(PhantomData<T>);

impl<T> SchemaColumn for ColumnType<T> {}
impl<S, Q, T> BindItem<S, Q> for ColumnType<T>
where
    S: Database,
    T: sqlx::Type<S>,
    Q: Query,
{
    fn bind_item(
        self,
        ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String + 'static {
        use sqlx::TypeInfo;
        let ty = T::type_info();
        let ty = ty.name().to_string();
        move |_| format!("{}", ty,)
    }
}

pub struct DefaultConstraint<ToBeAccepted, T>(
    ToBeAccepted,
    PhantomData<T>,
);

// impl<S, ToBeAccepted, T, Q, I> BindItem<S, Q, I>
//     for DefaultConstraint<ToBeAccepted, T>
// where
//     Q: Accept<ToBeAccepted, S>,
// {
//     fn column(
//         self,
//         ctx: &mut Q::Context1,
//     ) -> impl FnOnce(&mut Q::Context2) -> String {
//         let save = Q::accept(self.0, ctx);
//         |ctxr| format!("DEFAULT {}", save(ctxr))
//     }
// }
impl<A, T> Constraint for DefaultConstraint<A, T> {}
impl<S, ToBeAccepted, T, Q> BindItem<S, Q>
    for DefaultConstraint<ToBeAccepted, T>
where
    Q: Accept<ToBeAccepted, S>,
{
    fn bind_item(
        self,
        ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String + 'static {
        let save = Q::accept(self.0, ctx);
        |ctxr| format!("DEFAULT {}", save(ctxr))
    }
}

pub struct NotNull;

impl SchemaColumn for NotNull {}
impl<S, Q: Query> BindItem<S, Q> for NotNull
where
    S: Database,
{
    fn bind_item(
        self,
        _: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String + 'static {
        move |_| "NOT NULL".to_string()
    }
}

pub struct Or<T>(Vec<T>);

// impl<S, Q, T> BindItem<S, Q> for Or<T>
// where
//     T: 'static,
//     S: 'static,
//     Q: 'static,
//     Q::Context1: 'static,
//     Q: Query,
//     T: BindItem<S, Q>,
// {
//     fn bind_item(
//         self,
//         ctx: &mut <Q as Query>::Context1,
//     ) -> impl FnOnce(&mut <Q as Query>::Context2) -> String + 'static
//     {
//         let mut maps = vec![];
//         for each in self.0 {
//             // SAFETY:
//             // say ctx: `&'ctx mut Q::Context1` and `Q::Context1: 'ctx`
//             //
//             // bind_item retrun is 'static
//             // that means that when bind_item returns I'm guaranteed to
//             // not hold 'ctx and I can create new unique mut ref
//             //
//             // I think my reasoning is correct but I don't know why
//             // rust is not allowing this without unsafe
//             let res =
//                 each.bind_item(unsafe { &mut *(ctx as *mut _) });
//
//             let res: Box<
//                 dyn FnOnce(
//                     &mut <Q as Query>::Context2,
//                 ) -> String,
//             > = Box::new(res);
//             maps.push(res);
//         }
//         |ctx| {
//             maps.into_iter()
//                 .map(|e| e(ctx))
//                 .collect::<Vec<_>>()
//                 .join(" OR ")
//         }
//     }
// }

pub mod exports {
    pub use super::aliasing::SelectHelpers;
    pub use super::SelectHelpers2;

    use super::*;
    pub fn or<T>(input: Vec<T>) -> Or<T> {
        Or(input)
    }

    pub fn col<
        I: IdentSafety,
        In: ToOwned<Owned = I::Column>,
    >(
        name: In,
    ) -> Column<I> {
        Column {
            name: name.to_owned(),
        }
    }

    pub fn ft<I: IdentSafety>(
        name: I::Table,
    ) -> ForeignTable<I> {
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
