#![allow(unused)]
#![allow(deprecated)]
use serde::Deserialize;

pub mod delete_one;
pub mod get_all;
pub mod get_one;
pub mod insert_one;
pub mod update_one;

#[derive(Debug, Deserialize)]
pub struct Id {
    pub id: i64,
}
