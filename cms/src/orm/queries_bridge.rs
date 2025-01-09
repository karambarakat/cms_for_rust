use queries_for_sqlx::{
    ident_safety::PanicOnUnsafe, prelude::stmt as st,
    quick_query::QuickQuery,
};
use sqlx::Sqlite;

// todo: support 'impl Database'

pub type SelectSt =
    st::SelectSt<Sqlite, QuickQuery<'static>, PanicOnUnsafe>;

pub type InsertSt = st::InsertStOne<'static, Sqlite, ()>;

pub type UpdateSt =
    st::UpdateSt<Sqlite, QuickQuery<'static>, ()>;

pub type DeleteSt =
    st::DeleteSt<Sqlite, QuickQuery<'static>, ()>;
