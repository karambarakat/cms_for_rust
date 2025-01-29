use crate::execute_no_cache::ExecuteNoCacheUsingSelectTrait;
use crate::ident_safety::{self, PanicOnUnsafe};
use crate::{
    Accept, AcceptColIdent, IdentSafety, Query, QueryHandlers,
    Statement,
};
use crate::{AcceptTableIdent, BindItem};
use std::marker::PhantomData;

pub struct SelectSt<S, Q: Query, I: IdentSafety> {
    pub(crate) select_list:
        Vec<(Option<I::Table>, I::Column, Option<&'static str>)>,
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

impl<S, Q, I> ExecuteNoCacheUsingSelectTrait
    for SelectSt<S, Q, I>
where
    I: IdentSafety,
    Q: Query,
{
}

impl<S, Q, I> SelectSt<S, Q, I>
where
    Q: Query,
    I: IdentSafety,
{
    pub fn init<T: AsRef<str>>(from: T) -> Self
    where
        I: AcceptTableIdent<T>,
    {
        let ident_safety = I::init(Some(&from));
        SelectSt {
            select_list: Default::default(),
            where_clause: Default::default(),
            joins: vec![],
            order_by: Default::default(),
            limit: Default::default(),
            shift: Default::default(),
            ctx: Default::default(),
            from: I::into_table(from),
            ident_safety,
            _sqlx: PhantomData,
        }
    }
}

impl<S, Q, I> Statement<S, Q> for SelectSt<S, Q, I>
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

    fn _build(self) -> (String, Q::Output) {
        self.build()
    }
}

impl<S, Q, I> SelectSt<S, Q, I>
where
    Q: Query,
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
                let limit = Q::build_sql_part_back(ctx, limit);
                str.push_str(" LIMIT ");
                str.push_str(&limit);
            }

            if let Some(shift) = self.shift {
                let shift = Q::build_sql_part_back(ctx, shift);
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
        I: AcceptTableIdent<T>,
        I: AcceptColIdent<C>,
    {
        let i = I::into_col(item);
        let t = I::into_table(table);
        self.select_list.push((Some(t), i, Some(alias)));
    }

    pub fn select_scoped<T, C>(&mut self, table: T, item: C)
    where
        I: AcceptTableIdent<T>,
        I: AcceptColIdent<C>,
    {
        let i = I::into_col(item);
        let t = I::into_table(table);
        self.select_list.push((Some(t), i, None));
    }
    pub fn select<T>(&mut self, item: T)
    where
        I: AcceptColIdent<T>,
    {
        let i = I::into_col(item);
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
        I: AcceptColIdent<T>,
    {
        self.order_by.push((I::into_col(by), asc));
    }
}

impl<S, Q, I> SelectSt<S, Q, I>
where
    Q: QueryHandlers<S>,
    Q: Query,
    I: IdentSafety,
{
    pub fn offset<T>(&mut self, shift: T)
    where
        Q: Accept<T, S>,
        T: Send + 'static,
    {
        if self.shift.is_some() {
            panic!("limit has been set already");
        }

        let limit = Q::handle_accept(shift, &mut self.ctx);

        self.shift = Some(limit);
    }

    pub fn limit<T>(&mut self, limit: T)
    where
        Q: Accept<T, S>,
        T: Send + 'static,
    {
        if self.limit.is_some() {
            panic!("limit has been set already");
        }

        let limit = Q::handle_accept(limit, &mut self.ctx);

        self.limit = Some(limit);
    }
    pub fn where_<T>(&mut self, item: T)
    where
        T: BindItem<S, Q, I> + 'static,
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
