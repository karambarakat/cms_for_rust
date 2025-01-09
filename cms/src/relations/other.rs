use queries_for_sqlx::{
    ident_safety::PanicOnUnsafe,
    prelude::verbatim__warning__does_not_sanitize,
    quick_query::QuickQuery, select_st::SelectSt,
    SupportNamedBind,
};
use sqlx::Database;

use super::SelectStLimit;

pub struct BaseKeySet {
    pub vec: Vec<i64>,
}

impl<S> SelectStLimit<S> for BaseKeySet {
    fn limit(
        &self,
        st: &mut SelectSt<S, QuickQuery, PanicOnUnsafe>,
    ) where
        S: Database + SupportNamedBind,
    {
        st.where_(verbatim__warning__does_not_sanitize(
            self.vec
                .iter()
                .map(|e| format!("base_key = {}", e))
                .collect::<Vec<_>>()
                .join(" OR "),
        ));
    }
}

pub struct OneKey {
    pub id: i64,
}

impl<S> SelectStLimit<S> for OneKey {
    fn limit(
        &self,
        st: &mut SelectSt<S, QuickQuery, PanicOnUnsafe>,
    ) where
        S: Database + SupportNamedBind,
    {
        st.where_(verbatim__warning__does_not_sanitize(
            format!("base_key = {}", self.id),
        ));
    }
}
