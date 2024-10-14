
#![allow(unused)]

use std::collections::HashMap;

use crate::{ internal::*, ts};

#[derive(Debug, PartialEq)]
struct S1 {
    a: i32,
    b: i32,
}

#[derive(Debug, PartialEq)]
struct S2<T> {
    c: i32,
    d: T,
}

#[derive(Debug, PartialEq)]
struct S3<T>(String, T);

#[test]
fn test_str_to_my_val() {
    let gen_val = String::from("hello world");

    let debug_val = DebugOutput::String("hello world".to_string());

    assert_eq!(
        debug_val,
        str_to_debug(ParsingStream::new(&mut format!(
            "{:?}",
            gen_val
        )))
        .unwrap()
    );

    let gen_val = "hello world";

    let debug_val = DebugOutput::String("hello world".to_string());

    assert_eq!(
        debug_val,
        str_to_debug(ParsingStream::new(&mut format!(
            "{:?}",
            gen_val
        )))
        .unwrap()
    );

    let gen_val = ("hello world", 32);

    let debug_val = DebugOutput::Vec(
        [
            DebugOutput::String("hello world".to_string()),
            DebugOutput::Other("32".to_string()),
        ]
        .into(),
    );

    assert_eq!(
        debug_val,
        str_to_debug(ParsingStream::new(&mut format!(
            "{:?}",
            gen_val
        )))
        .unwrap()
    );

    let gen_val = [3, 4, 5];

    let debug_val = DebugOutput::Vec(
        [
            DebugOutput::Other("3".to_string()),
            DebugOutput::Other("4".to_string()),
            DebugOutput::Other("5".to_string()),
        ]
        .into(),
    );

    assert_eq!(
        debug_val,
        str_to_debug(ParsingStream::new(&mut format!(
            "{:?}",
            gen_val
        )))
        .unwrap()
    );



    let gen_val = [[3, 5], [4, 3], [3, 5]];

    let debug_val = DebugOutput::Vec(
        [
            DebugOutput::Vec(
                [
                    DebugOutput::Other("3".to_string()),
                    DebugOutput::Other("5".to_string()),
                ]
                .into(),
            ),
            DebugOutput::Vec(
                [
                    DebugOutput::Other("4".to_string()),
                    DebugOutput::Other("3".to_string()),
                ]
                .into(),
            ),
            DebugOutput::Vec(
                [
                    DebugOutput::Other("3".to_string()),
                    DebugOutput::Other("5".to_string()),
                ]
                .into(),
            ),
        ]
        .into(),
    );

    assert_eq!(
        debug_val,
        str_to_debug(ParsingStream::new(&mut format!(
            "{:?}",
            gen_val
        )))
        .unwrap()
    );




    let gen_val = S1 { a: 32, b: 22 };

    let debug_val = DebugOutput::Named(
        "S1".to_string(),
        Box::new(DebugOutput::Map(
            [
                (
                    "a".to_string(),
                    DebugOutput::Other("32".to_string()),
                ),
                (
                    "b".to_string(),
                    DebugOutput::Other("22".to_string()),
                ),
            ]
            .into(),
        )),
    );

    assert_eq!(
        debug_val,
        str_to_debug(ParsingStream::new(&mut format!(
            "{:?}",
            gen_val
        )))
        .unwrap()
    );



    let gen_val = S2 {
        c: 32,
        d: S2 {
            c: 4,
            d: S3(
                String::from("hello world"),
                Some((
                    "String",
                    S2 {
                        c: 0,
                        d: vec![
                            S1 { a: 133, b: 0 },
                            S1 { a: 93, b: 3 },
                        ],
                    },
                )),
            ),
        },
    };

        let debug_val = DebugOutput::Named(
                "S2".to_string(),
                Box::new(DebugOutput::Map(
                    [
                        (
                            "c".to_string(),
                            DebugOutput::Other("32".to_string()),
                        ),
                        (
                            "d".to_string(),
                            DebugOutput::Named(
                                "S2".to_string(),
                                Box::new(DebugOutput::Map(
                                    [
                                        (
                                            "c".to_string(),
                                            DebugOutput::Other("4".to_string()),
                                        ),
                                        (
                                            "d".to_string(),
                                            DebugOutput::Named(
                                                "S3".to_string(),
                                                Box::new(DebugOutput::Vec(
                                                    [
                                                        DebugOutput::String("hello world".to_string()),
                                                        DebugOutput::Named(
                                                            "Some".to_string(),
                                                            Box::new(DebugOutput::Vec([
                                                                DebugOutput::Vec([
                                                                    DebugOutput::String("String".to_string()),
                                                                    DebugOutput::Named(
                                                                        "S2".to_string(),
                                                                        Box::new(DebugOutput::Map(
                                                                            [
                                                                                (
                                                                                    "c".to_string(),
                                                                                    DebugOutput::Other("0".to_string()),
                                                                                ),
                                                                                (
                                                                                    "d".to_string(),
                                                                                    DebugOutput::Vec(
                                                                                        [
                                                                                            DebugOutput::Named(
                                                                                                "S1".to_string(),
                                                                                                Box::new(DebugOutput::Map(
                                                                                                    [
                                                                                                        (
                                                                                                            "a".to_string(),
                                                                                                            DebugOutput::Other("133".to_string()),
                                                                                                        ),
                                                                                                        (
                                                                                                            "b".to_string(),
                                                                                                            DebugOutput::Other("0".to_string()),
                                                                                                        ),
                                                                                                    ]
                                                                                                    .into(),
                                                                                                )),
                                                                                            ),
                                                                                            DebugOutput::Named(
                                                                                                "S1".to_string(),
                                                                                                Box::new(DebugOutput::Map(
                                                                                                    [
                                                                                                        (
                                                                                                            "a".to_string(),
                                                                                                            DebugOutput::Other("93".to_string()),
                                                                                                        ),
                                                                                                        (
                                                                                                            "b".to_string(),
                                                                                                            DebugOutput::Other("3".to_string()),
                                                                                                ),
                                                                                            ]
                                                                                            .into(),
                                                                                        )),
                                                                                    ),
                                                                                ]
                                                                                .into(),
                                                                            ),
                                                                        ),
                                                                    ]
                                                                    .into(),
                                                                )),
                                                            ),
                                                        ].into()),
                                                    ].into()))
                                                    )
                                                
                                            ]
                                            .into(),
                                        )),
                                    ),
                                ),
                            ]
                            .into(),
                        )),
                    ),
                ),
            ]
            .into(),
        )),
    );

    assert_eq!(
        debug_val,
        str_to_debug(ParsingStream::new(&mut format!(
            "{:?}",
            gen_val
        )))
        .unwrap()
    );
}
