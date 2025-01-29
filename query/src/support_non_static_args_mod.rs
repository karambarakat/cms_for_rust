use crate::{Accept, BindItem, IdentSafety};

pub trait Query<'s, 'q: 's>: Sized {
    type IdentSafety: IdentSafety;
    type SqlPart;
    type Context1: Default + 'q;
    type Context2: From<Self::Context1> + 'q;

    fn build_sql_part_back(
        ctx: &'s mut Self::Context2,
        from: Self::SqlPart,
    ) -> String;

    type Output;

    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&'s mut Self::Context2) -> String,
    ) -> (String, Self::Output);
}

pub trait QueryHandlers<'s, 'q: 's, S>: Query<'s, 'q> {
    fn handle_bind_item<T, I>(
        t: T,
        ctx: &'s mut <Self as Query<'s, 'q>>::Context1,
    ) -> <Self as Query<'s, 'q>>::SqlPart
    where
        T: BindItem<'s, 'q, S, Self, I> + 'static;

    fn handle_accept<T>(
        t: T,
        ctx: &'s mut <Self as Query<'s, 'q>>::Context1,
    ) -> <Self as Query<'s, 'q>>::SqlPart
    where
        T: 'static + Send,
        Self: Accept<'s, 'q, T, S>;
}

pub trait Statement<'s, 'q: 's, S, Q: Query<'s, 'q>> {
    fn deref_ctx(&'s self) -> &'s Q::Context1;
    fn deref_mut_ctx(&'s mut self) -> &'s mut Q::Context1;
    #[track_caller]
    fn _build(self) -> (String, Q::Output);
}

pub trait Accept<'s, 'q: 's, This, S>:
    QueryHandlers<'s, 'q, S> + Send
{
    fn accept(
        this: This,
        ctx1: &'s mut Self::Context1,
    ) -> impl FnOnce(&'s mut Self::Context2) -> String + 's + Send;
}

pub trait BindItem<'s, 'q, S, Q, I>
where
    <Q as Query<'s, 'q>>::Context1: 'q,
    Q: Query<'s, 'q>,
    'q: 's,
{
    fn bind_item(
        self,
        ctx: &'s mut Q::Context1,
    ) -> impl FnOnce(
        &'s mut <Q as Query<'s, 'q>>::Context2,
    ) -> String
           + 's;
}

mod expression_impls {
    impl<'s, 'q: 's, S, Q, Col, T1, I: IdentSafety>
        BindItem<'s, 'q, S, Q, I> for ColEq<Col, T1>
    where
        Q: Accept<'s, 'q, T1, S>,

        Col: NonBindItem<I = I>,
    {
        fn bind_item(
            self,
            ctx: &'s mut <Q as Query<'s, 'q>>::Context1,
        ) -> impl FnOnce(
            &'s mut <Q as Query<'s, 'q>>::Context2,
        ) -> String
               + 's
        where
            <Q as Query<'s, 'q>>::Context1: 'q,
        {
            let map = Q::accept(self.1, ctx);
            move |ctx| format!("{} = {}", self.0, map(ctx))
        }
    }

    impl<'s, 'q: 's, S, Q, T1, I> BindItem<'s, 'q, S, Q, I>
        for Or<T1>
    where
        Q: Query<'s, 'q>,
        T1: BindItem<'s, 'q, S, Q, I>,
        Q::Context2: 'static,
    {
        fn bind_item(
            self,
            ctx: &'s mut Q::Context1,
        ) -> impl FnOnce(&'s mut Q::Context2) -> String + 's
        where
            <Q as Query<'s, 'q>>::Context1: 'q,
        {
            let ptr = ctx as *mut _;

            let mut maps: Vec<Box<dyn FnOnce(_) -> _>> =
                Vec::new();
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

    impl<'s, 'q: 's, S, Q, T0, T1, I> BindItem<'s, 'q, S, Q, I>
        for All<(T0, T1)>
    where
        T0: 'q,
        T1: 'q,
        T0: BindItem<'s, 'q, S, Q, I>,
        T1: BindItem<'s, 'q, S, Q, I>,
        Q: Query<'s, 'q>,
        // I wonder if I can remove this requirement in the future?
        <Q as Query<'s, 'q>>::Context1: 'static,
    {
        fn bind_item(
            self,
            ctx: &'s mut <Q as Query<'s, 'q>>::Context1,
        ) -> impl FnOnce(
            &'s mut <Q as Query<'s, 'q>>::Context2,
        ) -> String
               + 's {
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
}

pub mod schema_items {
    use std::{fmt, marker::PhantomData};

    use sqlx::{Database, Type};

    use crate::{BindItem, IdentSafety, Query, SchemaColumn};

    impl<'s, 'q: 's, T, S, Q, I> BindItem<'s, 'q, S, Q, I>
        for ColumnType<T>
    where
        Q: Query<'s, 'q>,
        S: Database,
        T: Type<S> + 'static,
    {
        fn bind_item(
            self,
            ctx: &'s mut <Q as Query<'s, 'q>>::Context1,
        ) -> impl FnOnce(
            &mut <Q as Query<'s, 'q>>::Context2,
        ) -> String
               + 's
        where
            <Q as Query<'s, 'q>>::Context1: 'q,
        {
            move |_| self.display()
        }
    }
}

pub mod update_st {
    use std::marker::PhantomData;

    use sqlx::Database;

    use crate::{
        execute_no_cache::ExecuteNoCacheUsingSelectTrait,
        ident_safety::PanicOnUnsafe, returning::ReturningClause,
        Accept, AcceptColIdent, BindItem, IdentSafety, Query,
        QueryHandlers, Statement, SupportNamedBind,
        SupportReturning,
    };

    pub struct UpdateSt<
        's,
        'q,
        S,
        Q: Query<'s, 'q>,
        I: IdentSafety,
        R = (),
    > {
        pub(crate) sets: Vec<(I::Column, Q::SqlPart)>,
        pub(crate) where_clause: Vec<Q::SqlPart>,
        pub(crate) ctx: Q::Context1,
        pub(crate) table: I::Table,
        pub(crate) returning: R,
        pub(crate) _sqlx: PhantomData<S>,
    }

    impl<'s, 'q, S, Q: Query<'s, 'q>, I: IdentSafety, R>
        UpdateSt<'s, 'q, S, Q, I, R>
    {
        pub fn set_len(&self) -> usize {
            self.sets.len()
        }
    }

    impl<'s, 'q, S, Q: Query<'s, 'q>, I: IdentSafety, R>
        ExecuteNoCacheUsingSelectTrait
        for UpdateSt<'s, 'q, S, Q, I, R>
    {
    }

    impl<'s, 'q, S, Q> UpdateSt<'s, 'q, S, Q, ()>
    where
        Q: Query<'s, 'q>,
    {
        pub fn init(init: String) -> Self {
            UpdateSt {
                sets: Vec::new(),
                where_clause: Vec::new(),
                ctx: Default::default(),
                table: init.to_string(),
                returning: (),
                _sqlx: PhantomData,
            }
        }
    }
    impl<'s, 'q: 's, S, Q, I: IdentSafety, R>
        Statement<'s, 'q, S, Q> for UpdateSt<'s, 'q, S, Q, I, R>
    where
        S: Database + SupportNamedBind,
        Q: Query<'s, 'q>,
        R: ReturningClause,
    {
        fn deref_ctx(&self) -> &Q::Context1 {
            &self.ctx
        }
        fn deref_mut_ctx(&mut self) -> &mut Q::Context1 {
            &mut self.ctx
        }

        fn _build(
            self,
        ) -> (String, <Q as Query<'s, 'q>>::Output) {
            self.build()
        }
    }

    impl<'s, 'q: 's, S, R, Q, I> UpdateSt<'s, 'q, S, Q, I, R>
    where
        S: SupportNamedBind,
        S: Database,
        Q: Query<'s, 'q>,
        I: IdentSafety,
    {
        pub fn build(self) -> (String, Q::Output)
        where
            R: ReturningClause,
            S: Database + SupportNamedBind,
        {
            <Q as Query>::build_query(self.ctx, |ctx| {
                let mut str = String::from("UPDATE ");

                str.push_str(self.table.as_ref());

                str.push_str(" SET ");

                if self.sets.is_empty() {
                    panic!("empty set on update")
                }

                for (index, (column, value)) in
                    self.sets.into_iter().enumerate()
                {
                    if index != 0 {
                        str.push_str(", ");
                    }
                    str.push_str(column.as_ref());
                    str.push_str(" = ");
                    str.push_str(
                        &<Q as Query>::build_sql_part_back(
                            ctx, value,
                        ),
                    );
                }

                for (index, where_item) in
                    self.where_clause.into_iter().enumerate()
                {
                    if index == 0 {
                        str.push_str(" WHERE ");
                    } else {
                        str.push_str(" AND ");
                    }
                    str.push_str(
                        &<Q as Query>::build_sql_part_back(
                            ctx, where_item,
                        ),
                    );
                }

                str.push_str(&self.returning.returning());

                str
            })
        }
    }

    impl<'s, 'q, S, Q: Query<'s, 'q>, I: IdentSafety>
        UpdateSt<'s, 'q, S, Q, I, ()>
    {
        pub fn returning_<R>(
            self,
            cols: Vec<R>,
        ) -> UpdateSt<'s, 'q, S, Q, I, Vec<R>>
        where
            S: SupportReturning,
        {
            UpdateSt {
                sets: self.sets,
                where_clause: self.where_clause,
                ctx: self.ctx,
                table: self.table,
                returning: cols,
                _sqlx: PhantomData,
            }
        }
        #[deprecated]
        pub fn returning(
            self,
            cols: Vec<&'static str>,
        ) -> UpdateSt<'s, 'q, S, Q, I, Vec<&'static str>>
        {
            UpdateSt {
                sets: self.sets,
                where_clause: self.where_clause,
                ctx: self.ctx,
                table: self.table,
                returning: cols,
                _sqlx: PhantomData,
            }
        }
    }

    impl<'s, 'q: 's, S, Q, I, R> UpdateSt<'s, 'q, S, Q, I, R>
    where
        Q: QueryHandlers<'s, 'q, S>,
        I: IdentSafety,
    {
        pub fn set<C, T>(&'s mut self, column: C, value: T)
        where
            I: AcceptColIdent<C>,
            Q: Accept<'s, 'q, T, S>,
            T: Send + 'static,
        {
            let part = Q::handle_accept(value, &mut self.ctx);
            self.sets.push((I::into_col(column), part));
        }

        pub fn where_<Item>(&'s mut self, item: Item)
        where
            Item: BindItem<'s, 'q, S, Q, I> + 'static,
        {
            let item = Q::handle_bind_item(item, &mut self.ctx);
            self.where_clause.push(item);
        }
    }
}

pub mod select_st {
    use crate::execute_no_cache::ExecuteNoCacheUsingSelectTrait;
    use crate::ident_safety::{self, PanicOnUnsafe};
    use crate::{
        Accept, AcceptColIdent, IdentSafety, Query,
        QueryHandlers, Statement,
    };
    use crate::{AcceptTableIdent, BindItem};
    use std::marker::PhantomData;

    pub struct SelectSt<'q, S, Q: Query<'q>, I: IdentSafety> {
        pub(crate) select_list: Vec<(
            Option<I::Table>,
            I::Column,
            Option<&'static str>,
        )>,
        pub(crate) where_clause: Vec<Q::SqlPart>,
        pub(crate) joins: Vec<(&'static str, join)>,
        pub(crate) order_by: Vec<(I::Column, bool)>,
        pub(crate) limit: Option<Q::SqlPart>,
        pub(crate) shift: Option<Q::SqlPart>,
        pub(crate) ctx: Q::Context1,
        pub(crate) from: I::Table,
        pub(crate) ident_safety: I,
        pub(crate) _sqlx: PhantomData<(S, I)>,
    }

    impl<'q, S, Q, I> ExecuteNoCacheUsingSelectTrait
        for SelectSt<'q, S, Q, I>
    where
        I: IdentSafety,
        Q: Query<'q>,
    {
    }

    impl<'q, S, Q: Query<'q>>
        SelectSt<'q, S, Q, <Q as Query<'q>>::IdentSafety>
    {
        pub fn init<T: AsRef<str>>(from: T) -> Self
        where
            <Q as Query<'q>>::IdentSafety: AcceptTableIdent<T>,
        {
            let ident_safety = Q::IdentSafety::init(Some(&from));
            SelectSt {
            select_list: Default::default(),
            where_clause: Default::default(),
            joins: vec![],
            order_by: Default::default(),
            limit: Default::default(),
            shift: Default::default(),
            ctx: Default::default(),
            from: <Q::IdentSafety as AcceptTableIdent<T>>::into_table(
                from,
            ),
            ident_safety,
            _sqlx: PhantomData,
        }
        }
    }

    impl<'q, S, Q, I> Statement<'q, S, Q> for SelectSt<'q, S, Q, I>
    where
        Q: Query<'q, IdentSafety = I>,
        I: IdentSafety,
    {
        fn deref_ctx(&self) -> &Q::Context1 {
            &self.ctx
        }
        fn deref_mut_ctx(&mut self) -> &mut Q::Context1 {
            &mut self.ctx
        }

        fn _build(self) -> (String, Q::Output) {
            self.build()
        }
    }

    impl<'q, S, Q, I> SelectSt<'q, S, Q, I>
    where
        Q: Query<'q, IdentSafety = I>,
        I: IdentSafety,
    {
        pub fn build(self) -> (String, Q::Output) {
            Q::build_query(self.ctx, |ctx| {
                let mut str = String::from("SELECT ");

                if self.select_list.len() == 0 {
                    panic!("select list is empty");
                }

                for (index, item) in
                    self.select_list.into_iter().enumerate()
                {
                    if index != 0 {
                        str.push_str(", ");
                    }
                    if let Some(table) = item.0 {
                        str.push_str(table.as_ref());
                        str.push_str(".");
                    }
                    str.push_str(item.1.as_ref());
                    if let Some(alias) = item.2 {
                        str.push_str(" AS ");
                        str.push_str(alias);
                    }
                }

                str.push_str(" FROM ");
                str.push_str(self.from.as_ref());

                for join in self.joins.into_iter() {
                    let join = format!(
                        " {} {} ON {}.{} = {}.{}",
                        join.0,
                        join.1.on_table,
                        join.1.on_table,
                        join.1.on_column,
                        self.from.as_ref(),
                        join.1.local_column,
                    );
                    str.push_str(&join);
                }

                for (index, item) in
                    self.where_clause.into_iter().enumerate()
                {
                    let item = Q::build_sql_part_back(ctx, item);
                    if item.is_empty() {
                        tracing::error!(
                            "item should not be empty {}",
                            item
                        );
                        continue;
                    }
                    if index == 0 {
                        str.push_str(" WHERE ");
                    } else {
                        str.push_str(" AND ");
                    }

                    str.push_str(&item);
                }

                if self.order_by.len() != 0 {
                    str.push_str(" ORDER BY ");
                    for (index, (by, asc)) in
                        self.order_by.into_iter().enumerate()
                    {
                        if index != 0 {
                            str.push_str(", ");
                        }
                        str.push_str(by.as_ref());
                        if !asc {
                            str.push_str(" DESC");
                        }
                    }
                }

                if let Some(limit) = self.limit {
                    let limit =
                        Q::build_sql_part_back(ctx, limit);
                    str.push_str(" LIMIT ");
                    str.push_str(&limit);
                }

                if let Some(shift) = self.shift {
                    let shift =
                        Q::build_sql_part_back(ctx, shift);
                    str.push_str(" OFFSET ");
                    str.push_str(&shift);
                }

                str.push_str(";");
                str
            })
        }

        pub fn select_aliased<T, C>(
            &mut self,
            table: T,
            item: C,
            alias: &'static str,
        ) where
            Q::IdentSafety: AcceptTableIdent<T>,
            Q::IdentSafety: AcceptColIdent<C>,
        {
            let i = Q::IdentSafety::into_col(item);
            let t = Q::IdentSafety::into_table(table);
            self.select_list.push((Some(t), i, Some(alias)));
        }

        pub fn select_scoped<T, C>(&mut self, table: T, item: C)
        where
            Q::IdentSafety: AcceptTableIdent<T>,
            Q::IdentSafety: AcceptColIdent<C>,
        {
            let i = Q::IdentSafety::into_col(item);
            let t = Q::IdentSafety::into_table(table);
            self.select_list.push((Some(t), i, None));
        }
        pub fn select<T>(&mut self, item: T)
        where
            Q::IdentSafety: AcceptColIdent<T>,
        {
            let i = Q::IdentSafety::into_col(item);
            self.select_list.push((None, i, None));
        }

        pub fn left_join(&mut self, j: join) {
            if self
                .joins
                .iter()
                .find(|e| e.1.on_table == j.on_table)
                .is_some()
            {
                panic!(
                    "table {} has been joined already",
                    j.on_table
                );
            }
            self.joins.push(("LEFT JOIN", j));
        }

        // pub fn join(&mut self, join: Join<I>) {
        //     if self
        //         .joins
        //         .iter()
        //         .find(|e| {
        //             e.on_table.as_ref() == join.on_table.as_ref()
        //         })
        //         .is_some()
        //     {
        //         panic!(
        //             "table {} has been joined already",
        //             join.on_table.as_ref()
        //         );
        //     }
        //
        //     self.joins.push(join);
        // }

        pub fn order_by<T>(&mut self, by: T, asc: bool)
        where
            Q::IdentSafety: AcceptColIdent<T>,
        {
            self.order_by
                .push((Q::IdentSafety::into_col(by), asc));
        }
    }

    impl<'s, 'q: 's, S, Q, I> SelectSt<'q, S, Q, I>
    where
        Q: QueryHandlers<'s, 'q, S>,
        Q: Query<'q, IdentSafety = I>,
        I: IdentSafety,
    {
        pub fn offset<T>(&'s mut self, shift: T)
        where
            Q: Accept<'s, 'q, T, S>,
            T: Send + 'static,
        {
            if self.shift.is_some() {
                panic!("limit has been set already");
            }

            let limit = Q::handle_accept(shift, &mut self.ctx);

            self.shift = Some(limit);
        }

        pub fn limit<T>(&'s mut self, limit: T)
        where
            Q: Accept<'s, 'q, T, S>,
            T: Send + 'static,
        {
            if self.limit.is_some() {
                panic!("limit has been set already");
            }

            let limit = Q::handle_accept(limit, &mut self.ctx);

            self.limit = Some(limit);
        }
        pub fn where_<T>(&'s mut self, item: T)
        where
            T: BindItem<'s, 'q, S, Q, I> + 'static,
        {
            let item = Q::handle_bind_item(item, &mut self.ctx);

            self.where_clause.push(item);
        }
    }

    #[allow(non_camel_case_types)]
    pub struct join {
        pub on_table: String,
        pub on_column: String,
        pub local_column: String,
    }

    pub mod order_by {
        pub const ASC: bool = true;
        pub const DESC: bool = false;
    }
}

pub mod quick_query {
    use crate::QueryHandlers;

    use std::{marker::PhantomData, ops::Deref, sync::Arc};

    use sqlx::{database::HasArguments, Database, Encode, Type};

    use crate::{
        ident_safety::PanicOnUnsafe, Accept, BindItem, Query,
        SupportNamedBind,
    };

    pub struct QuickQuery<'q, S>(PhantomData<(&'q (), S)>);

    pub struct QuickQueryCtx1<'q, S: Database> {
        size: usize,
        arg: <S as HasArguments<'q>>::Arguments,
        noop: (),
    }

    impl<'q, S: Database> Default for QuickQueryCtx1<'q, S> {
        fn default() -> Self {
            QuickQueryCtx1 {
                size: 0,
                arg: Default::default(),
                noop: Default::default(),
            }
        }
    }

    impl<'q, S: Database> From<QuickQueryCtx1<'q, S>> for () {
        fn from(this: QuickQueryCtx1<'q, S>) -> Self {
            this.noop
        }
    }

    impl<'s, 'q: 's, S: Database + SupportNamedBind>
        Query<'s, 'q> for QuickQuery<'q, S>
    {
        type SqlPart = String;
        type Context1 = QuickQueryCtx1<'q, S>;
        type Context2 = ();
        fn build_sql_part_back(
            _: &'s mut Self::Context2,
            from: Self::SqlPart,
        ) -> String {
            from
        }
        type Output = <S as HasArguments<'q>>::Arguments;
        fn build_query(
            mut ctx1: Self::Context1,
            f: impl FnOnce(&'s mut Self::Context2) -> String,
        ) -> (String, Self::Output) {
            // I wonder if I can remove this in the future
            let noop =
                unsafe { &mut *(&mut ctx1.noop as *mut _) };
            let strr = f(noop);
            (strr, ctx1.arg)
        }
    }

    impl<'q, 's, S> QueryHandlers<'s, 'q, S> for QuickQuery<'q, S>
    where
        'q: 's,
        S: Database + SupportNamedBind,
        // needed because the S in this impl may not match the S in Query impl:
        Self: Query<
            's,
            'q,
            SqlPart = String,
            Context1 = QuickQueryCtx1<'q, S>,
            Context2 = (),
        >,
    {
        fn handle_accept<T>(
            t: T,
            ctx: &'s mut Self::Context1,
        ) -> Self::SqlPart
        where
            Self: Accept<'s, 'q, T, S>,
        {
            let cc = &mut ctx.noop as &mut ();
            let cc2 = unsafe { &mut *(cc as *mut _) };
            Self::accept(t, ctx)(cc2)
        }
        fn handle_bind_item<T, I>(
            t: T,
            ctx: &'s mut Self::Context1,
        ) -> Self::SqlPart
        where
            T: BindItem<'s, 'q, S, Self, I> + 'static,
        {
            let cc = &mut ctx.noop as &mut ();
            let cc2 = unsafe { &mut *(cc as *mut _) };
            t.bind_item(ctx)(cc2)
        }
    }

    #[cfg(not(feature = "flexible_accept_impl"))]
    impl<'s, 'q: 's, S, T> Accept<'s, 'q, T, S> for QuickQuery<'q, S>
    where
        S: Database + SupportNamedBind,
        T: for<'e> Encode<'e, S> + Type<S> + Send + 'q,
    {
        fn accept(
            this: T,
            ctx1: &'s mut Self::Context1,
        ) -> impl FnOnce(&'s mut Self::Context2) -> String + 's + Send
        {
            use sqlx::Arguments;
            ctx1.arg.add(this);
            ctx1.size += 1;
            let len = ctx1.size;
            move |_| format!("${}", len)
        }
    }

    #[cfg(feature = "flexible_accept_impl")]
    impl<'q, S, T> Accept<bind<T>, S> for QuickQuery<'q, S>
    where
        S: Database + SupportNamedBind,
        T: for<'e> Encode<'e, S> + Type<S> + Send + 'q,
    {
        fn accept(
            this: ToBeAccepted,
            ctx1: &mut Self::Context1,
        ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
        {
            use sqlx::Arguments;
            ctx1.1.add(this.0);
            ctx1.0 += 1;
            let len = ctx1.0;
            move |_| format!("${}", len)
        }
    }

    #[cfg(feature = "flexible_accept_impl")]
    impl<'q, S, ToBeAccepted, T> Accept<ToBeAccepted, S>
        for QuickQuery<'q, S>
    where
        S: Database + SupportNamedBind,
        ToBeAccepted: FnOnce() -> T,
        T: for<'e> Encode<'e, S> + Type<S> + Send + 'q,
    {
        fn accept(
            this: ToBeAccepted,
            ctx1: &mut Self::Context1,
        ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
        {
            use sqlx::Arguments;
            ctx1.1.add(this());
            ctx1.0 += 1;
            let len = ctx1.0;
            move |_| format!("${}", len)
        }
    }
}

pub mod positional_query {
    use std::ops::IndexMut;
    use std::{marker::PhantomData, mem::take};

    use sqlx::{
        database::HasArguments, prelude::Type, Arguments,
        Database, Encode,
    };

    use crate::{
        ident_safety::PanicOnUnsafe, Accept, BindItem, Query,
        QueryHandlers,
    };
    use crate::{AcceptColIdent, AcceptTableIdent, IdentSafety};

    pub struct PositionalQuery<'q, S>(PhantomData<(&'q (), S)>);

    pub trait Stored<S> {
        fn bind(
            self: Box<Self>,
            ctx: &mut <S as HasArguments<'static>>::Arguments,
        ) where
            S: Database;
    }

    impl<S, T> Stored<S> for T
    where
        S: Database,
        T: Type<S> + for<'q> Encode<'q, S> + 'static + Send,
    {
        fn bind(
            self: Box<Self>,
            ctx: &mut <S as HasArguments<'static>>::Arguments,
        ) where
            S: Database,
        {
            ctx.add(*self);
        }
    }

    pub struct NoOpIdentSafety;
    impl IdentSafety for NoOpIdentSafety {
        type Table = String;

        type Column = String;

        fn check_other<T: AsRef<str>>(any_: T) {}

        fn init<T: AsRef<str>>(on_table: Option<&T>) -> Self {
            NoOpIdentSafety
        }
    }

    impl AcceptColIdent<&'static str> for NoOpIdentSafety {
        fn into_col(this: &'static str) -> Self::Column {
            this.to_string()
        }
    }

    impl AcceptTableIdent<&'static str> for NoOpIdentSafety {
        fn into_table(this: &'static str) -> Self::Table {
            this.to_string()
        }
    }

    pub struct PositionalCtx2<S: Database> {
        pub(crate) back: Vec<Option<Box<dyn Stored<S>>>>,
        pub(crate) arg: <S as HasArguments<'static>>::Arguments,
    }

    impl<S: Database> From<Vec<Option<Box<dyn Stored<S>>>>>
        for PositionalCtx2<S>
    {
        fn from(back: Vec<Option<Box<dyn Stored<S>>>>) -> Self {
            PositionalCtx2 {
                back,
                arg: Default::default(),
            }
        }
    }

    impl<'s, S> Query<'s, 'static> for PositionalQuery<'static, S>
    where
        S: Database,
    {
        type IdentSafety = NoOpIdentSafety;

        type SqlPart =
            Box<dyn FnOnce(&'s mut Self::Context2) -> String>;

        type Context1 = Vec<Option<Box<dyn Stored<S>>>>;

        type Context2 = PositionalCtx2<S>;

        fn build_sql_part_back(
            ctx: &'s mut Self::Context2,
            mut from: Self::SqlPart,
        ) -> String {
            from(ctx)
        }

        type Output = <S as HasArguments<'static>>::Arguments;

        fn build_query(
            ctx1: Self::Context1,
            f: impl FnOnce(&'s mut Self::Context2) -> String,
        ) -> (String, Self::Output) {
            // let mut ctx2 = (ctx1, Default::default());
            let mut ctx2 = PositionalCtx2::from(ctx1);
            let ptr = &mut ctx2 as *mut _;
            let str = f(unsafe { &mut *ptr });
            let output = ctx2.arg;
            return (str, output);
        }
    }

    impl<'s, S> QueryHandlers<'s, 'static, S>
        for PositionalQuery<'static, S>
    where
        Self: Query<
            's,
            'static,
            SqlPart = Box<
                dyn FnOnce(&'s mut Self::Context2) -> String,
            >,
            Context1 = Vec<Option<Box<dyn Stored<S>>>>,
            Context2 = PositionalCtx2<S>,
        >,
        S: Database,
    {
        fn handle_bind_item<T, I>(
            t: T,
            ctx: &'s mut Self::Context1,
        ) -> Self::SqlPart
        where
            T: BindItem<'s, 'static, S, Self, I> + 'static,
        {
            Box::new(move |ctx| {
                let ptr = &mut ctx.back;
                let ptr2 = unsafe { &mut *(ptr as *mut _) };
                let b = t.bind_item(ptr2);
                b(ctx)
            })
        }

        fn handle_accept<T>(
            t: T,
            ctx: &'s mut Self::Context1,
        ) -> Self::SqlPart
        where
            T: Send + 'static,
            Self: Accept<'s, 'static, T, S>,
        {
            Box::new(move |ctx| {
                let cc = &mut ctx.back as &mut _;
                let cc2 = unsafe { &mut *(cc as *mut _) };
                let bring_back = Self::accept(t, cc2);
                bring_back(ctx)
            })
        }
    }

    impl<'s, S, T> Accept<'s, 'static, T, S>
        for PositionalQuery<'static, S>
    where
        S: Database,
        T: Type<S> + for<'q> Encode<'q, S> + 'static + Send,
    {
        fn accept(
            this: T,
            ctx1: &'s mut Self::Context1,
        ) -> impl FnOnce(&'s mut Self::Context2) -> String + 's + Send
        {
            ctx1.push(Some(Box::new(this)));
            let len = ctx1.len();

            move |ctx2| {
                let bring_back = ctx2
                    .back
                    .get_mut(len - 1)
                    .map(|e| e.take())
                    .flatten()
                    .expect(
                        "should be bound and taken only once",
                    );

                bring_back.bind(&mut ctx2.arg);

                "?".to_string()
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::sync::Mutex;

        use sqlx::{
            encode::IsNull,
            sqlite::{
                SqliteArgumentValue, SqliteArguments,
                SqliteTypeInfo, SqliteValue,
            },
            Encode, Sqlite, Type,
        };

        use crate::{
            ident_safety::{define_schema, PanicOnUnsafe},
            positional_query::NoOpIdentSafety,
            prelude::col,
            select_st::SelectSt,
        };

        use super::PositionalQuery;

        struct StringAlias(String);

        static BIND_ORDER: Mutex<Vec<String>> =
            Mutex::new(Vec::new());

        impl<'q> Encode<'q, Sqlite> for StringAlias {
            fn encode_by_ref(
                &self,
                buf: &mut Vec<SqliteArgumentValue<'q>>,
            ) -> IsNull {
                BIND_ORDER.lock().unwrap().push(self.0.clone());

                <String as Encode<'q, Sqlite>>::encode_by_ref(
                    &self.0, buf,
                )
            }
            fn encode(
                self,
                buf: &mut <Sqlite as sqlx::database::HasArguments<
                'q,
            >>::ArgumentBuffer,
            ) -> IsNull
            where
                Self: Sized,
            {
                BIND_ORDER.lock().unwrap().push(self.0.clone());

                <String as Encode<'q, Sqlite>>::encode(
                    self.0, buf,
                )
            }
        }

        impl Type<Sqlite> for StringAlias {
            fn type_info() -> SqliteTypeInfo {
                todo!()
            }
        }

        #[test]
        fn positional_query_figure_out_order() {
            let mut st = SelectSt::<
                Sqlite,
                PositionalQuery<'static, Sqlite>,
                NoOpIdentSafety,
            >::init("Todo");

            define_schema(&[("Todo", &["id", "title"])]);

            st.select(col("*"));
            st.where_(
                col("id").eq(StringAlias("1".to_string())),
            );
            st.offset(StringAlias("2".to_string()));

            let (str, arg) = st.build();

            drop(arg);

            assert_eq!(
                str,
                "SELECT * FROM Todo WHERE id = ? OFFSET ?;"
            );

            let bind_order = BIND_ORDER.lock().unwrap().clone();

            assert_eq!(
                bind_order,
                vec!["1".to_string(), "2".to_string()]
            );

            BIND_ORDER.lock().unwrap().drain(..);

            // even when we call offset before where,
            // PositionalQuery should know to reorder them

            let mut st = SelectSt::<
                Sqlite,
                PositionalQuery<'static, Sqlite>,
                NoOpIdentSafety,
            >::init("Todo");

            define_schema(&[("Todo", &["id", "title"])]);

            st.select(col("*"));
            st.offset(StringAlias("2".to_string()));
            st.where_(
                col("id").eq(StringAlias("1".to_string())),
            );

            let (str, arg) = st.build();

            drop(arg);

            assert_eq!(
                str,
                "SELECT * FROM Todo WHERE id = ? OFFSET ?;"
            );

            let bind_order = BIND_ORDER.lock().unwrap().clone();

            assert_eq!(
                bind_order,
                vec!["1".to_string(), "2".to_string()]
            );
        }
    }
}

mod impl_execute_no_cache {
    impl<'q, S, Q, T> ExecuteNoCache<'q, S, Q> for T
    where
        S: Database,
        T: Statement<'q, S, Q> + ExecuteNoCacheUsingSelectTrait,
        Q: Query<
            'q,
            Output = <S as HasArguments<'q>>::Arguments,
        >,
    {
        #[track_caller]
        fn build(
            self,
        ) -> (String, <S as HasArguments<'q>>::Arguments)
        {
            <Self as Statement<S, Q>>::_build(self)
        }
    }
}

pub mod delete_st {
    use std::marker::PhantomData;

    use sqlx::Database;

    use crate::{
        execute_no_cache::ExecuteNoCacheUsingSelectTrait,
        ident_safety::PanicOnUnsafe, returning::ReturningClause,
        AcceptTableIdent, BindItem, IdentSafety, Query,
        QueryHandlers, Statement, SupportNamedBind,
    };

    pub struct DeleteSt<
        'q,
        S,
        Q: Query<'q>,
        I: IdentSafety,
        R = (),
    > {
        pub(crate) where_clause: Vec<Q::SqlPart>,
        pub(crate) ctx: Q::Context1,
        pub(crate) table: I::Table,
        pub(crate) returning: R,
        pub(crate) _sqlx: PhantomData<S>,
    }

    impl<'q, S, Q, I: IdentSafety, R>
        ExecuteNoCacheUsingSelectTrait
        for DeleteSt<'q, S, Q, I, R>
    where
        Q: Query<'q>,
    {
    }

    impl<'q, S, Q, I> DeleteSt<'q, S, Q, I>
    where
        Q: Query<'q>,
        I: IdentSafety,
    {
        pub fn init<M>(init: M) -> Self
        where
            I: AcceptTableIdent<M>,
        {
            DeleteSt {
                where_clause: Vec::new(),
                ctx: Default::default(),
                table: I::into_table(init),
                returning: (),
                _sqlx: PhantomData,
            }
        }
    }
    impl<'q, S, Q, I, R> Statement<'q, S, Q>
        for DeleteSt<'q, S, Q, I, R>
    where
        R: ReturningClause,
        S: Database + SupportNamedBind,
        Q: Query<'q>,
        I: IdentSafety,
    {
        fn deref_ctx(&self) -> &<Q as Query<'q>>::Context1 {
            &self.ctx
        }
        fn deref_mut_ctx(
            &mut self,
        ) -> &mut <Q as Query<'q>>::Context1 {
            &mut self.ctx
        }
        fn _build(self) -> (String, <Q as Query<'q>>::Output) {
            self.build()
        }
    }

    impl<'q, S, Q: Query<'q>, I: IdentSafety> DeleteSt<'q, S, Q, I> {
        pub fn returning(
            self,
            returning: Vec<&'static str>,
        ) -> DeleteSt<'q, S, Q, I, Vec<&'static str>> {
            DeleteSt {
                returning,
                where_clause: self.where_clause,
                ctx: self.ctx,
                table: self.table,
                _sqlx: PhantomData,
            }
        }
    }

    impl<'s, 'q: 's, S, Q: Query<'q>, I: IdentSafety>
        DeleteSt<'q, S, Q, I>
    {
        pub fn where_<W>(&'s mut self, item: W)
        where
            W: BindItem<'s, 'q, S, Q, I> + 'static,
            Q: QueryHandlers<'s, 'q, S>,
        {
            let item = Q::handle_bind_item(item, &mut self.ctx);
            self.where_clause.push(item);
        }
    }

    impl<'q, S, Q, R, I> DeleteSt<'q, S, Q, I, R>
    where
        S: SupportNamedBind,
        S: Database,
        R: ReturningClause,
        I: IdentSafety,
        Q: Query<'q>,
    {
        pub fn build(self) -> (String, Q::Output) {
            <Q as Query>::build_query(self.ctx, |ctx| {
                let mut str = String::from("DELETE FROM ");

                str.push_str(self.table.as_ref());

                for (index, where_item) in
                    self.where_clause.into_iter().enumerate()
                {
                    if index == 0 {
                        str.push_str(" WHERE ");
                    } else {
                        str.push_str(" AND ");
                    }
                    str.push_str(
                        &<Q as Query>::build_sql_part_back(
                            ctx, where_item,
                        ),
                    );
                }

                str.push_str(&self.returning.returning());

                str
            })
        }
    }
}

pub mod create_table_st {
    use std::{fmt::Display, marker::PhantomData};

    use sqlx::{database::HasArguments, Database};

    use crate::{
        execute_no_cache::ExecuteNoCacheUsingSelectTrait,
        ident_safety::PanicOnUnsafe, AcceptTableIdent, BindItem,
        Constraint, IdentSafety, Query, QueryHandlers,
        SchemaColumn, Statement,
    };

    #[derive(Debug)]
    pub struct CreateTableSt<'q, S, Q: Query<'q>, I: IdentSafety> {
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

    impl<'q, S, Q: Query<'q>, I: IdentSafety>
        ExecuteNoCacheUsingSelectTrait
        for CreateTableSt<'q, S, Q, I>
    {
    }

    impl<'q, S, Q, I> CreateTableSt<'q, S, Q, I>
    where
        Q: Query<'q>,
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

    impl<'q, S, Q, I> Statement<'q, S, Q>
        for CreateTableSt<'q, S, Q, I>
    where
        Q: Query<'q>,
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
                    let item =
                        Q::build_sql_part_back(ctx, col.1);
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

    impl<'s, 'q: 's, S, Q, I> CreateTableSt<'q, S, Q, I>
    where
        I: IdentSafety,
        Q: Query<'q>,
        S: Database,
    {
        pub fn verbatim(&mut self, verbatim: &str) {
            self.verbatim.push(verbatim.to_string());
        }
        pub fn column<C>(&'s mut self, name: &str, constraint: C)
        where
            C: SchemaColumn<S> + 'static,
            C: BindItem<'s, 'q, S, Q, I>,
            Q: QueryHandlers<'s, 'q, S>,
        {
            let item =
                Q::handle_bind_item(constraint, &mut self.ctx);
            self.columns.push((name.to_string(), item));
        }
        pub fn constraint<C>(&'s mut self, constraint: C)
        where
            C: Constraint + 'static,
            C: BindItem<'s, 'q, S, Q, I>,
            Q: QueryHandlers<'s, 'q, S>,
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
}

pub mod clonable_query_impls {
    impl<'s, 'q: 's, S: Database + SupportNamedBind>
        Query<'s, 'q> for ClonablQuery<'q, S>
    {
        type IdentSafety = PanicOnUnsafe;
        type SqlPart = String;
        type Context1 = ClonableCtx1<'q, S>;
        type Context2 = ();
        fn build_sql_part_back(
            _: &mut Self::Context2,
            from: Self::SqlPart,
        ) -> String {
            from
        }
        type Output = <S as HasArguments<'q>>::Arguments;
        fn build_query(
            mut ctx1: Self::Context1,
            f: impl FnOnce(&'s mut Self::Context2) -> String,
        ) -> (String, Self::Output) {
            let noop =
                unsafe { &mut *(&mut ctx1.noop as *mut _) };
            let strr = f(noop);
            (strr, ctx1.arg)
        }
    }

    impl<'s, 'q, S> QueryHandlers<'s, 'q, S> for ClonablQuery<'q, S>
    where
        'q: 's,
        S: Database + SupportNamedBind,
        // needed because the S in this impl may not match the S in Query impl:
        Self: Query<
            's,
            'q,
            SqlPart = String,
            Context1 = ClonableCtx1<'q, S>,
            Context2 = (),
        >,
    {
        fn handle_accept<T>(
            t: T,
            ctx: &'s mut Self::Context1,
        ) -> Self::SqlPart
        where
            Self: Accept<'s, 'q, T, S>,
        {
            let noop = &mut ctx.noop as &mut ();
            let noop_ptr = unsafe { &mut *(noop as *mut _) };
            Self::accept(t, ctx)(noop_ptr)
        }
        fn handle_bind_item<T, I>(
            t: T,
            ctx: &'s mut Self::Context1,
        ) -> Self::SqlPart
        where
            T: BindItem<'s, 'q, S, Self, I> + 'static,
        {
            let noop = &mut ctx.noop as &mut ();
            let noop_ptr = unsafe { &mut *(noop as *mut _) };
            t.bind_item(ctx)(noop_ptr)
        }
    }

    #[cfg(not(feature = "flexible_accept_impl"))]
    impl<'s, 'q, S, T> Accept<'s, 'q, T, S> for ClonablQuery<'q, S>
    where
        'q: 's,
        S: Database + SupportNamedBind,
        T: for<'e> Encode<'e, S> + Type<S> + Send + 'q + Clone,
    {
        fn accept(
            this: T,
            ctx1: &'s mut Self::Context1,
        ) -> impl FnOnce(&'s mut Self::Context2) -> String + 's + Send
        {
            use sqlx::Arguments;
            let cloned = this.clone();
            ctx1.back.push(Box::new(cloned));
            ctx1.arg.add(this);
            ctx1.size += 1;
            let len = ctx1.size;
            move |_| format!("${}", len)
        }
    }
}
