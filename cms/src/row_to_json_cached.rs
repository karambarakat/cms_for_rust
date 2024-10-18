#![allow(unused)]
use queries_for_sqlx::string_query::row_into_json::RowToJson;
use serde_json::Value;
use sqlx::sqlite::SqliteRow;
use sqlx::Column;
use sqlx::Database;
use sqlx::Row;
use sqlx::Sqlite;
use sqlx::TypeInfo;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::DefaultHasher;
use std::hash::RandomState;
use std::sync::Arc;
use std::sync::{Mutex, RwLock};

// static RW_STORE: RwLock<HashMap<Entry, Box<dyn RowToJson>>> =
//     RwLock::new(Vec::new());
static LOCK: RwLock<()> = RwLock::new(());

#[derive(Hash, Debug, PartialEq, Eq, Clone)]
pub struct Entry {
    /// col name, is_null, col type
    cols: Vec<(String, bool, String)>,
}

static RW_STORE_SQLITE: RwLock<
    Vec<(
        Entry,
        Arc<dyn Fn(&Entry, SqliteRow) -> Value + Send + Sync>,
    )>,
> = RwLock::new(Vec::new());

pub fn sqlite_row(
) -> impl FnMut(SqliteRow) -> Result<Value, sqlx::Error> + Send {
    let mut cache_once = None;
    let mut entry_cache = None;
    move |row| {
        if cache_once.is_none() {
            let lock = RW_STORE_SQLITE.read().unwrap();
            let entry = Entry {
                cols: row
                    .columns()
                    .iter()
                    .map(|e| {
                        (
                            e.name().to_string(),
                            e.type_info().is_null(),
                            e.type_info().name().to_string(),
                        )
                    })
                    .collect(),
            };

            let found_cached = lock.iter().find_map(|e| {
                if e.0 == entry {
                    cache_once = Some(e.1.clone());
                    Some(())
                } else {
                    None
                }
            });

            if found_cached.is_none() {
                drop(lock);
                let mut wlock = RW_STORE_SQLITE.write().unwrap();
                wlock.push((
                    entry.clone(),
                    Arc::new(move |entry: &Entry, row| {
                        <Sqlite as RowToJson>::row_to_json(
                            row,
                            &entry.cols,
                        )
                        .into()
                    }),
                ));
                let ok =
                    Arc::clone(&wlock.last_mut().unwrap().1);
                drop(wlock);
                cache_once = Some(ok)
            }

            entry_cache = Some(entry);
        }

        Ok(cache_once.as_ref().cloned().unwrap()(
            &entry_cache.as_ref().unwrap(),
            row,
        ))
    }
}

mod dyn_row {
    use std::marker::PhantomData;

    use serde_json::Value;
    use sqlx::ColumnIndex;
    use sqlx::Database;
    use sqlx::Decode;
    use sqlx::Row;
    use sqlx::Type;

    pub trait DynRow {
        fn db_name(&self) -> &str;
        fn get(&self, name: &str, ty: &dyn DynType) -> Value;
    }

    pub trait DynType {}

    impl<S, T> DynType for PhantomData<(S, T)>
    where
        S: Database,
        T: Type<S> + for<'d> Decode<'d, S>,
    {
    }

    impl<T> DynRow for T
    where
        T: Row,
        for<'s> &'s str: ColumnIndex<T>,
    {
        fn db_name(&self) -> &str {
            T::Database::NAME
        }

        fn get(&self, name: &str, ty: &dyn DynType) -> Value {
            todo!()
        }
    }
}
