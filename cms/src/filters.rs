use queries_for_sqlx::prelude::*;
use sqlx::Sqlite;

use crate::traits::Resource;

use super::{
    operations::select_many::Pagination,
    queries_bridge::SelectSt,
};

pub trait AgnosticFilter: Sync + Send {
    fn on_select(self, st: &mut SelectSt<Sqlite>);

    #[inline]
    fn into_filter(self) -> ImplFilter<Self>
    where
        Self: Sized,
    {
        ImplFilter(self)
    }
}

pub struct ImplFilter<T>(pub T);

impl<C, T> Filters<C> for ImplFilter<T>
where
    T: AgnosticFilter,
{
    fn on_select(self, st: &mut SelectSt<Sqlite>) {
        self.0.on_select(st);
    }
}

pub trait Filters<C>: Sync + Send {
    fn on_select(self, st: &mut SelectSt<Sqlite>);
}

pub struct ById(pub i64);
pub fn by_id(id: i64) -> ById {
    ById(id)
}

impl<C: Resource<Sqlite>> Filters<C> for ById {
    fn on_select(self, st: &mut SelectSt<Sqlite>) {
        st.where_(scoped(C::table_name(), "id").eq(self.0));
    }
}

impl AgnosticFilter for Pagination {
    fn on_select(self, st: &mut SelectSt<Sqlite>) {
        let offset = ((self.page - 1) * self.page_size);
        let limit = self.page_size;

        st.offset(offset);
        st.limit(limit);
    }
}

pub struct FilterLike<T> {
    _phantom: std::marker::PhantomData<T>,
}

pub struct Eq {}
