use std::marker::PhantomData;

use joins::{Join, JoinType};

use crate::execute_no_cache::ExecuteNoCacheUsingSelectTrait;
use crate::sql_part::{
    AcceptToSqlPart, ToSqlPart, WhereItemToSqlPart,
};
use crate::{
    Accept, InitStatement, Query, Statement, TakeParts,
};
use crate::{SelectItem, WhereItem};

pub struct SelectSt<S, Q: Query<S>> {
    pub(crate) select_list: Vec<String>,
    pub(crate) where_clause: Vec<Q::SqlPart>,
    pub(crate) joins: Vec<Join<&'static str>>,
    pub(crate) order_by: Vec<(&'static str, bool)>,
    pub(crate) limit: Option<Q::SqlPart>,
    pub(crate) shift: Option<Q::SqlPart>,
    pub(crate) ctx: Q::Context1,
    pub(crate) from: &'static str,
    pub(crate) _sqlx: PhantomData<S>,
}

impl<S, Q> ExecuteNoCacheUsingSelectTrait for SelectSt<S, Q> where
    Q: Query<S>
{
}

impl<S, Q> InitStatement<Q> for SelectSt<S, Q>
where
    Q: Query<S>,
{
    type Init = &'static str;
    fn init(from: &'static str) -> SelectSt<S, Q> {
        SelectSt {
            select_list: Default::default(),
            where_clause: Default::default(),
            joins: Default::default(),
            order_by: Default::default(),
            limit: Default::default(),
            shift: Default::default(),
            ctx: Default::default(),
            from,
            _sqlx: PhantomData,
        }
    }
}

impl<S, Q> Statement<S, Q> for SelectSt<S, Q>
where
    Q: Query<S>,
{
    fn deref_ctx(&self) -> &Q::Context1 {
        &self.ctx
    }
    fn deref_mut_ctx(&mut self) -> &mut Q::Context1 {
        &mut self.ctx
    }

    fn _build(self) -> (String, <Q as Query<S>>::Output) {
        self.build()
    }
}

impl<S, Q> SelectSt<S, Q>
where
    Q: Query<S>,
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
                str.push_str(&item);
            }

            str.push_str(" FROM ");
            str.push_str(&self.from);

            for join in self.joins.into_iter() {
                let join = format!(
                    " {} {} ON {}.{} = {}.{}",
                    join.ty.to_string(),
                    join.on_table,
                    join.on_table,
                    join.on_column,
                    self.from,
                    join.local_column,
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
                    str.push_str(by);
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

    pub fn select(
        &mut self,
        item: impl SelectItem<S> + 'static,
    ) {
        self.select_list.push(item.select_item());
    }

    pub fn join(&mut self, join: impl JoinType<S> + 'static) {
        let join = join.join_ty();
        if self
            .joins
            .iter()
            .find(|e| e.on_table == join.on_table)
            .is_some()
        {
            panic!(
                "table {} has been joined already",
                join.on_table
            );
        }

        self.joins.push(join);
    }

    pub fn offset<T>(&mut self, shift: T)
    where
        Q: Accept<T, S>,
        AcceptToSqlPart<T>: ToSqlPart<Q, S>,
    {
        if self.shift.is_some() {
            panic!("limit has been set already");
        }

        let limit =
            AcceptToSqlPart(shift).to_sql_part(&mut self.ctx);

        self.shift = Some(limit);
    }

    pub fn limit<T>(&mut self, limit: T)
    where
        Q: Accept<T, S>,
        AcceptToSqlPart<T>: ToSqlPart<Q, S>,
    {
        if self.limit.is_some() {
            panic!("limit has been set already");
        }

        let limit =
            AcceptToSqlPart(limit).to_sql_part(&mut self.ctx);

        self.limit = Some(limit);
    }

    pub fn order_by(&mut self, by: &'static str, asc: bool) {
        self.order_by.push((by, asc));
    }

    pub fn where_<T>(&mut self, item: T)
    where
        T: WhereItem<S, Q> + 'static,
        WhereItemToSqlPart<T>: ToSqlPart<Q, S>,
    {
        let item =
            WhereItemToSqlPart(item).to_sql_part(&mut self.ctx);

        self.where_clause.push(item);
    }
    // pub fn where_(
    //     &mut self,
    //     item: impl WhereItem<S, Q> + 'static,
    // )
    // {
    //     let item = Q::handle_where_item(item, &mut self.ctx);
    //
    //     self.where_clause.push(item);
    // }
}

#[deprecated = "in favor of AcceptToSqlPart"]
pub struct HandleAccept<T, S, Q>(pub T, pub PhantomData<(S, Q)>);

/// this trait will be removed once I figure out universal
/// way to accept all Q: Accept<S, T>
#[deprecated = "in favor of AcceptToSqlPart"]
pub trait HandleAcceptIsWorking {
    type SqlPart;
    type Ctx;
    fn to_sql_part(self, ctx: &mut Self::Ctx) -> Self::SqlPart;
}

pub mod joins {
    pub mod join_type {
        use super::{Join, JoinType};

        pub struct Left;

        impl<S> JoinType<S> for Join<Left> {
            fn join_ty(self) -> Join<&'static str> {
                Join {
                    ty: "LEFT JOIN",
                    on_table: self.on_table,
                    on_column: self.on_column,
                    local_column: self.local_column,
                }
            }
        }
    }

    pub trait JoinType<S> {
        fn join_ty(self) -> Join<&'static str>;
    }

    pub struct Join<J> {
        pub ty: J,
        pub on_table: &'static str,
        pub on_column: &'static str,
        pub local_column: &'static str,
    }
}

pub mod order_by {
    pub const ASC: bool = true;
    pub const DESC: bool = false;
}

pub mod exports {
    pub use super::joins::{Join, JoinType};
}
