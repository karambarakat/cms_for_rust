use queries_for_sqlx::{
    ident_safety::PanicOnUnsafe, prelude::*,
    quick_query::QuickQuery,
};
use sqlx::Sqlite;

use super::{
    operations::select_many::Pagination,
    queries_bridge::SelectSt, Collection, HasCol,
};

pub trait AgnosticFilter: Sync + Send {
    fn on_select(self, st: &mut SelectSt);

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
    fn on_select(self, st: &mut SelectSt) {
        self.0.on_select(st);
    }
}

pub trait Filters<C>: Sync + Send {
    fn on_select(self, st: &mut SelectSt);
}

pub struct ById(pub i64);
pub fn by_id(id: i64) -> ById {
    ById(id)
}

impl<C: Collection> Filters<C> for ById {
    fn on_select(
        self,
        st: &mut stmt::SelectSt<
            Sqlite,
            QuickQuery,
            PanicOnUnsafe,
        >,
    ) {
        st.where_(
            ft(C::table_name1().to_string())
                .col("id".to_string())
                .eq(move || self.0),
        );
    }
}

impl AgnosticFilter for Pagination {
    fn on_select(self, st: &mut SelectSt) {
        let offset = ((self.page - 1) * self.page_size);
        let limit = self.page_size;

        st.offset(move || offset);
        st.limit(move || limit);
    }
}

pub struct FilterLike<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<C, T: Send + Sync> Filters<C> for FilterLike<T>
where
    T: HasCol<C, This = String>,
{
    fn on_select(self, st: &mut SelectSt) {}
}
