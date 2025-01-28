use queries_for_sqlx::{
    clonable_query::ClonablQuery, ident_safety::PanicOnUnsafe, prelude::stmt as st, quick_query::QuickQuery
};
// todo: support 'impl Database'

pub type SelectSt<S> =
    st::SelectSt<S, QuickQuery<S>, PanicOnUnsafe>;

pub type InsertSt<S> = st::InsertStOne<'static, S, ()>;

pub type UpdateSt<S> =
    st::UpdateSt<S, QuickQuery<S>, PanicOnUnsafe, ()>;

pub type DeleteSt<S> =
    st::DeleteSt<S, QuickQuery<S>, PanicOnUnsafe, ()>;

pub type CreatTableSt<S> =
    queries_for_sqlx::create_table_st::CreateTableSt<
        S,
        ClonablQuery<'static, S>,
        PanicOnUnsafe,
    >;
