#![allow(unused)]

use core::fmt;
use std::ops::Not;

use better_testing::{
    displaying, expect,
    ord_mod::{or, ord},
    Expect, Similar, ToBe,
};

#[test]
fn test_expect() {
    expect(&Some("hello world")).to_be(&Some("hello world"));

    expect(&None).not().to_be(&Some("hello world"));
    expect(&430).similar(or(
        ord().greater_than(30).less_than(10),
        ord().greater_than(100).less_than(700),
    ));

    let err = std::panic::catch_unwind(|| {
        expect(&None).to_be(&Some("hello world"));
    });

    if err.is_err().not() {
        panic!("should fail")
    }

    let similar_to = "helo {

    }

[
 w
]";

    expect("helo { } [ w ]").similar(displaying(similar_to, ()));

    // i'm not sure about this one, I might remove the impls
    // associated with these (impl Similar<&'static str> for str)
    // and (impl Similar<&'static str> for String)
    expect("helo { } [ w ]").similar(similar_to);

    expect("helo { } [ w ]").displaying(similar_to, ());
}
