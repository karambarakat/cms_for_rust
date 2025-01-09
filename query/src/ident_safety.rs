use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
};

pub trait IdentSafety {
    type Table: AsRef<str>;
    type Column: AsRef<str>;
    #[track_caller]
    fn check_table(this: &Self::Table) {}
    #[track_caller]
    fn check_column(table: &Self::Table, this: &Self::Column) {}
}

pub struct PanicOnUnsafe;

lazy_static::lazy_static! {
    static ref IDENTS: RwLock<(HashSet<String>, HashMap<String, HashSet<String>>)> = RwLock::new((HashSet::new(), HashMap::new()));
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
    #[track_caller]
    fn check_table(this: &Self::Table) {
        let this: &str = this.as_ref();
        if !IDENTS.read().unwrap().0.contains(this) {
            panic!(
                "InjectionRisk: table {} is not defined",
                this
            );
        }
    }

    #[track_caller]
    fn check_column(table: &Self::Table, this: &Self::Column) {
        let this: &str = this.as_ref();
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
    }
}

pub struct StaticStr;
pub struct ImplTrait;
pub struct NoSafety;
