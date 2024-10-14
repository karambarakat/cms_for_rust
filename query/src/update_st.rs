use std::marker::PhantomData;

use sqlx::Database;

use crate::{
    returning::ReturningClause,
    select_st::{HandleAccept, HandleAcceptIsWorking},
    Accept, Query, SupportNamedBind, WhereItem,
};

pub struct UpdateSt<S, Q: Query<S>, R = ()> {
    pub(crate) sets: Vec<(&'static str, Q::SqlPart)>,
    pub(crate) where_clause: Vec<Q::SqlPart>,
    pub(crate) ctx: Q::Context1,
    pub(crate) table: &'static str,
    pub(crate) returning: R,
    pub(crate) _sqlx: PhantomData<S>,
}

impl<'q, S, R, Q> UpdateSt<S, Q, R>
where
    S: SupportNamedBind,
    S: Database,
    Q: Query<S>,
{
    pub fn _build(self) -> (String, Q::Output)
    where
        R: ReturningClause,
        S: Database + SupportNamedBind,
    {
        <Q as Query<S>>::build_query(self.ctx, |ctx| {
            let mut str = String::from("UPDATE ");

            str.push_str(self.table);

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
                str.push_str(column);
                str.push_str(" = ");
                str.push_str(
                    &<Q as Query<S>>::build_sql_part_back(
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
                    &<Q as Query<S>>::build_sql_part_back(
                        ctx, where_item,
                    ),
                );
            }

            str.push_str(&self.returning.returning());

            str
        })
    }
}

impl<S, Q: Query<S>> UpdateSt<S, Q, ()> {
    pub fn returning(
        self,
        cols: Vec<&'static str>,
    ) -> UpdateSt<S, Q, Vec<&'static str>> {
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
impl<S, Q: Query<S>, R> UpdateSt<S, Q, R> {
    pub fn set<T>(&mut self, column: &'static str, value: T)
    where
        Q: Accept<T, S>,
        HandleAccept<T, S, Q>: HandleAcceptIsWorking<
            SqlPart = Q::SqlPart,
            Ctx = Q::Context1,
        >,
    {
        let part = HandleAccept(value, PhantomData)
            .to_sql_part(&mut self.ctx);
        self.sets.push((column, part));
    }

    pub fn where_(
        &mut self,
        item: impl WhereItem<S, Q> + 'static,
    ) {
        let item = Q::handle_where_item(item, &mut self.ctx);
        self.where_clause.push(item);
    }
}
