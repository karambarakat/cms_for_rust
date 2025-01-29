use std::collections::HashMap;

use cms_macros::prepared_statement;

pub struct Input {
    value: String,
}

prepared_statement!(
    fn prepared(inn: Input, some_mut: &mut String) {
        if inn.value.is_empty() {
            if inn.value.is_empty() {
                some_mut.push_str("1");
            } else {
                some_mut.push_str("2");
            }
        } else {
            some_mut.push_str("2");
        }
    }
);

#[derive(Hash, PartialEq, Eq)]
enum SecondIf {
    If {},
    Else {},
}

#[derive(Hash, PartialEq, Eq)]
enum FirstIf {
    If { second_if: SecondIf },
    Else {},
}

#[derive(Hash, PartialEq, Eq)]
struct HashedInput {
    first_if: FirstIf,
}

// struct stmt {}
//
// impl stmt {
//     fn new(input: Input) -> Self {
//         let hi = HashedInput::from_ref(&input);
//
//         MAP.get_or_insert_with(hi, || {
//             let mut stmt = stmt {};
//             prepared(hi, &mut stmt)
//             stmt
//         })
//     }
// }

// lazy_static::lazy_static! {
//     static ref MAP : HashMap<HashedInput, stmt> = Default::defualt();
// }

#[test]
fn test_prep() {
    assert_eq!(debugs::COUNT, 2);

    // stmt::new(Input {
    //     value: "true".into(),
    // })
}
