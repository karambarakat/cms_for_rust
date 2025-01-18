use queries_for_sqlx::{
    ident_safety::PanicOnUnsafe, prelude::stmt as st,
    quick_query::QuickQuery,
};
// todo: support 'impl Database'

pub type SelectSt<S> =
    st::SelectSt<S, QuickQuery<'static, S>, PanicOnUnsafe>;

pub type InsertSt<S> = st::InsertStOne<'static, S, ()>;

pub type UpdateSt<S> =
    st::UpdateSt<S, QuickQuery<'static, S>, ()>;

pub type DeleteSt<S> =
    st::DeleteSt<S, QuickQuery<'static, S>, ()>;

