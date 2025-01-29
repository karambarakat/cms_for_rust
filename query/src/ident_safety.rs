use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
};

use crate::{AcceptColIdent, AcceptTableIdent, IdentSafety};

#[derive(Default)]
pub struct PanicOnUnsafe;

lazy_static::lazy_static! {
    static ref IDENTS: RwLock<(HashSet<String>, HashMap<String, HashSet<String>>)> = RwLock::new((HashSet::new(), HashMap::new()));
}

pub fn append_schema(table: &str, columns: &[&str]) {
    let mut idents = IDENTS.write().unwrap();
    idents.0.insert(table.to_string());
    let columns =
        columns.iter().map(|x| x.to_string()).collect();
    idents.1.insert(table.to_string(), columns);
}

pub fn define_schema(columns: &[(&str, &[&str])]) {
    let mut idents = IDENTS.write().unwrap();
    for (table, columns) in columns {
        idents.0.insert(table.to_string());
        let columns =
            columns.iter().map(|x| x.to_string()).collect();
        idents.1.insert(table.to_string(), columns);
    }
}

impl IdentSafety for PanicOnUnsafe {
    type Table = String;
    type Column = String;
    fn check_other<T: AsRef<str>>(any_: T) {
        let any_: &str = any_.as_ref();
        if any_.contains('\'') {
            panic!(
                "InjectionRisk: found single quote in {}",
                any_
            );
        }
    }
    fn init<T: AsRef<str>>(on_table: Option<&T>) -> Self {
        if let Some(on_table) = on_table {
            check_table(on_table.as_ref());
        }
        Self::default()
    }
}

pub type NoOp = ();

impl IdentSafety for NoOp {
    type Table = String;

    type Column = String;

    fn check_other<T: AsRef<str>>(any_: T) {}

    fn init<T: AsRef<str>>(on_table: Option<&T>) -> Self {}
}

#[track_caller]
pub fn check_column(table: Option<&str>, this: &str) {
    let this: &str = this.as_ref();

    if let Some(table) = table {
        let table: &str = table.as_ref();
        if !IDENTS.read().unwrap().0.contains(table) {
            panic!(
                "InjectionRisk: table {} is not defined",
                table
            );
        }
        if !IDENTS
            .read()
            .unwrap()
            .1
            .get(table)
            .unwrap()
            .contains(this)
        {
            panic!(
                "InjectionRisk: column {}.{} is not defined",
                table, this
            );
        }
    } else {
    }
}

#[track_caller]
pub fn check_table(this: &str) {
    let this: &str = this.as_ref();
    if !IDENTS.read().unwrap().0.contains(this) {
        panic!("InjectionRisk: table {} is not defined", this);
    }
}

impl AcceptTableIdent<&str> for PanicOnUnsafe {
    #[inline]
    #[track_caller]
    fn into_table(this: &str) -> String {
        check_table(this);
        this.to_owned()
    }
}

impl AcceptColIdent<&str> for PanicOnUnsafe {
    #[inline]
    #[track_caller]
    fn into_col(this: &str) -> String {
        check_column(None, this);
        this.to_owned()
    }
}

impl AcceptTableIdent<&String> for PanicOnUnsafe {
    #[inline]
    #[track_caller]
    fn into_table(this: &String) -> Self::Table {
        check_table(this);
        this.clone()
    }
}

impl AcceptColIdent<&String> for PanicOnUnsafe {
    #[inline]
    #[track_caller]
    fn into_col(this: &String) -> Self::Column {
        check_column(None, this);
        this.clone()
    }
}

impl AcceptTableIdent<String> for PanicOnUnsafe {
    fn into_table(this: String) -> String {
        check_table(&this);
        this
    }
}

impl AcceptColIdent<String> for PanicOnUnsafe {
    fn into_col(this: String) -> String {
        check_column(None, &this);
        this
    }
}

pub struct StaticStr;
pub struct ImplTrait;
pub struct NoSafety;
