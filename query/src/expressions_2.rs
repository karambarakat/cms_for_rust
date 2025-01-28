use std::fmt::{self, Display};

use crate::{
    Accept, AcceptColIdent, BindItem, IdentSafety, NonBindItem,
    Query, QueryHandlers,
};

pub struct ScopedCol<Q: IdentSafety>(
    Option<Q::Table>,
    Q::Column,
);

impl<I> AcceptColIdent<ScopedCol<I>> for I
where
    I: IdentSafety<Column = String>,
{
    fn into_col(
        this: ScopedCol<I>,
    ) -> <I as IdentSafety>::Column {
        format!("{}", this)
    }
}

impl<I> NonBindItem for ScopedCol<I>
where
    I: IdentSafety,
{
    type I = I;
}

pub struct Alias<Q: IdentSafety>(ScopedCol<Q>, String);

impl<I: IdentSafety> ScopedCol<I> {
    pub fn eq<T1>(self, value: T1) -> ColEq<Self, T1> {
        ColEq(self, value)
    }
    pub fn alias(self, alias: &str) -> Alias<I> {
        I::check_other(alias);
        Alias(self, alias.to_string())
    }
}

impl<I: IdentSafety> fmt::Display for ScopedCol<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(table) => write!(
                f,
                "{}.{}",
                table.as_ref(),
                self.1.as_ref()
            ),
            None => write!(f, "{}", self.1.as_ref()),
        }
    }
}

impl<I: IdentSafety> fmt::Display for Alias<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} AS {}", self.0, self.1)
    }
}

pub struct ColEq<Col, T1>(Col, T1);

#[cfg(not(feature = "support_non_static_args"))]
impl<S, Q, Col, T1, I: IdentSafety> BindItem<S, Q, I>
    for ColEq<Col, T1>
where
    Q: Accept<T1, S>,

    Col: NonBindItem<I = I>,
{
    fn bind_item(
        self,
        ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String + 'static {
        let map = Q::accept(self.1, ctx);
        move |ctx| format!("{} = {}", self.0, map(ctx))
    }
}

pub struct Or<T1>(pub Vec<T1>);

#[cfg(not(feature = "support_non_static_args"))]
impl<S, Q, T1, I> BindItem<S, Q, I> for Or<T1>
where
    Q: Query,
    T1: BindItem<S, Q, I>,
    Q::Context2: 'static,
{
    fn bind_item(
        self,
        ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String + 'static {
        let ptr = ctx as *mut _;

        let mut maps: Vec<Box<dyn FnOnce(_) -> _>> = Vec::new();
        for item in self.0 {
            // SAFETY:
            // say ctx: `&'ctx mut Q::Context1` and `Q::Context1: 'ctx`
            //
            // bind_item retrun is 'static
            // that means that when bind_item's return is guaranteed to
            // not hold 'ctx and I can create new unique mut ref
            //
            // I think my reasoning is correct but I don't know why
            // rust is not allowing this without unsafe
            let map = item.bind_item(unsafe { &mut *ptr });
            maps.push(Box::new(map));
        }

        move |ctx| {
            let ptr = ctx as *mut _;
            maps.into_iter()
                .map(|map| map(unsafe { &mut *ptr }))
                .collect::<Vec<_>>()
                .join(" OR ")
        }
    }
}

pub mod schema_items {
    use std::{fmt, marker::PhantomData};

    use sqlx::{Database, Type};

    use crate::{BindItem, IdentSafety, Query, SchemaColumn};

    pub struct ColumnType<T>(PhantomData<(T)>);

    impl<T, S, Q, I> BindItem<S, Q, I> for ColumnType<T>
    where
        Q: Query,
        S: Database,
        T: Type<S> + 'static,
    {
        fn bind_item(
            self,
            ctx: &mut Q::Context1,
        ) -> impl FnOnce(&mut Q::Context2) -> String + 'static
        {
            move |_| self.display()
        }
    }
    impl<T, S> SchemaColumn<S> for ColumnType<T>
    where
        S: Database,
        T: Type<S>,
    {
        fn display(&self) -> String {
            use sqlx::TypeInfo;

            let ty = T::type_info();
            let ty = ty.name();
            ty.to_string()
        }
    }

    pub fn col_type<T>() -> ColumnType<T> {
        ColumnType(PhantomData)
    }
}

pub mod schema_items_for_tupe {
    use crate::{BindItem, Query, SchemaColumn};

    pub struct All<T>(pub T);

    impl<S, Q, T0, T1, I> BindItem<S, Q, I>
        for All<(T0, T1)>
    where
        T0: BindItem<S, Q, I>,
        T1: BindItem<S, Q, I>,
        Q: Query,
    {
        fn bind_item(
            self,
            ctx: &mut <Q as Query>::Context1,
        ) -> impl FnOnce(
            &mut Q ::Context2,
        ) -> String
               + 'static {
            let ptr = ctx as *mut _;
            let this = self.0;
            // SAFETY:
            // say ctx: `&'ctx mut Q::Context1` and `Q::Context1: 'ctx`
            //
            // bind_item retrun is 'static
            // that means that when bind_item's return is guaranteed to
            // not hold 'ctx and I can create new unique mut ref
            //
            // I think my reasoning is correct but I don't know why
            // rust is not allowing this without unsafe
            let b0 = this.0.bind_item(unsafe { &mut *ptr });
            let b1 = this.1.bind_item(unsafe { &mut *ptr });
            |ctxb| {
                let ptr = ctxb as *mut _;
                format!(
                    "{} {}",
                    b0(unsafe { &mut *ptr }),
                    b1(unsafe { &mut *ptr }),
                )
            }
        }
    }

    impl<S, T0, T1> SchemaColumn<S> for All<(T0, T1)> {
        fn display(&self) -> String {
            "".to_string()
        }
    }

    pub fn all<T>(t: T) -> All<T> {
        All(t)
    }
}

pub mod exports {
    use crate::{
        AcceptColIdent, AcceptTableIdent, IdentSafety, Query,
    };

    pub fn or<T1>(items: Vec<T1>) -> super::Or<T1> {
        super::Or(items)
    }
    pub fn col<I: IdentSafety, T>(name: T) -> super::ScopedCol<I>
    where
        I: AcceptColIdent<T>,
    {
        super::ScopedCol(None, I::into_col(name))
    }
    pub fn scoped<I, T, N>(
        table: T,
        name: N,
    ) -> super::ScopedCol<I>
    where
        I: AcceptTableIdent<T>,
        I: AcceptColIdent<N>,
    {
        super::ScopedCol(
            Some(I::into_table(table)),
            I::into_col(name),
        )
    }
}
