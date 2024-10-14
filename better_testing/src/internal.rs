#![allow(unused)]
use std::collections::HashSet;
use std::iter::Peekable;
use std::ops::Sub;
use std::{collections::HashMap, ops::Not};
use std::fmt;
use std::mem::take;

#[derive(PartialEq)]
pub enum DebugOutput {
    Map(HashMap<String, DebugOutput>),
    Vec(Vec<DebugOutput>),
    Named(String, Box<DebugOutput>),
    String(String),
    Other(String),
}

impl fmt::Debug for DebugOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DebugOutput::Map(map) => {
                if map.len() == 0 || (map.len() == 1 && map.keys().collect::<Vec<_>>()[0] == "") {
                    return write!(f, "{{}}");
                }
                write!(f, "{{")?;
                for (k, v) in map {
                    write!(f, "{:?}: {:?}", k, v)?;
                    write!(f, ", ")?;
                }
                write!(f, "}}")
            }
            DebugOutput::Vec(vec) => {
                if vec.len() == 1 {
                    write!(f, "{:?}", vec[0])?;
                    return Ok(())
                }
                write!(f, "[")?;
                for v in vec {
                    write!(f, "{:?}, ", v)?;
                }
                write!(f, "]")
            }
            DebugOutput::Named(_, value) => {
                write!(f, "{:?}", value)
            }
            DebugOutput::String(s) => {
                write!(f, "{:?}", s)
            }
            DebugOutput::Other(s) => {
                write!(f, "{}", s)
            }
        }
    }
}

pub struct ParsingStream<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl fmt::Debug for ParsingStream<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!(
            "{}",
            self.chars.clone().collect::<String>()
        ))
    }
}

impl<'a> ParsingStream<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            chars: s.chars().peekable(),
        }
    }
    pub fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }
    pub fn next(&mut self) -> Option<char> {
        self.chars.next()
    }
    pub fn skip_str_into_key(&mut self, key: &mut String, next : char)  {
        if next == '"' {
            // val.push(next);
            while let Some(c) = self.next() {
                if c == '"' {
                    break;
                }
                if c == '\\' && self.peek() == Some(&'"') {
                    key.push(self.next().unwrap());
                    continue;
                }
                key.push(c);
            }
        }
    }
    pub fn skip_str(&mut self, val: &mut String, next: char) {
        if next == '"' {
            val.push(next);
            while let Some(c) = self.next() {
                if c == '"' {
                    val.push(c);
                    break;
                }
                val.push(c);
                if c == '\\' && self.peek() == Some(&'"') {
                    val.push(self.next().unwrap());
                }
            }
        }
    }
    pub fn skip<const O: char, const C: char>(
        &mut self,
        val: &mut String,
        next: char,
    ) {
        if next == O {
            val.push(next);
            let mut op = 1;

            while let Some(c) = self.next() {
                if c == O {
                    op += 1;
                } else if c == C {
                    op -= 1;
                }
                val.push(c);
                if op == 0 {
                    break;
                }
            }
        }
    }
}

pub fn str_to_debug(
    mut input: ParsingStream<'_>,
) -> Result<DebugOutput, &'static str> {
    if let Some('"') = input.peek() {
        let mut val = String::new();
        input.next();
        while let Some(next) = input.next() {
            if next == '"' {
                break;
            }
            if next == '\\' {
                val.push(next);
                val.push(input.next().unwrap());
            } else {
                val.push(next);
            }
        }
        return Ok(DebugOutput::String(val));
    }

    if let Some('(') = input.peek() {
        let mut val = String::new();
        let mut res = Vec::new();
        input.next();

        while let Some(next) = input.next() {
            input.skip::<'(', ')'>(&mut val, next);
            input.skip::<'{', '}'>(&mut val, next);
            input.skip::<'[', ']'>(&mut val, next);
            input.skip_str(&mut val, next);

            if next == ' ' {
                continue;
            }

            match next {
                ')' => {
                    res.push(str_to_debug(ParsingStream::new(
                        &mut take(&mut val),
                    ))?);
                }
                ',' => {
                    res.push(str_to_debug(ParsingStream::new(
                        &mut take(&mut val),
                    ))?);
                }
                _ => {
                    if next == ' ' && val.is_empty() {
                        continue;
                    }

                    val.push(next);
                }
            }
        }

        return Ok(DebugOutput::Vec(res));
    }

    // [] should behave the exact same as ()
    if let Some('[') = input.peek() {
        let mut val = String::new();
        let mut res = Vec::new();
        input.next();

        while let Some(next) = input.next() {
            input.skip::<'(', ')'>(&mut val, next);
            input.skip::<'{', '}'>(&mut val, next);
            input.skip::<'[', ']'>(&mut val, next);
            input.skip_str(&mut val, next);

            if next == ' ' {
                continue;
            }

            match next {
                ']' => {
                    res.push(str_to_debug(ParsingStream::new(
                        &mut take(&mut val),
                    ))?);
                }
                ',' => {
                    res.push(str_to_debug(ParsingStream::new(
                        &mut take(&mut val),
                    ))?);
                }
                _ => {
                    if next == ' ' && val.is_empty() {
                        continue;
                    }
                    val.push(next);
                }
            }
        }

        return Ok(DebugOutput::Vec(res));
    }

    if let Some('{') = input.peek() {
        let mut key = String::new();
        let mut val = String::new();
        let mut into_key = true;
        let mut res = HashMap::new();
        input.next();

        while let Some(next) = input.next() {
            input.skip::<'(', ')'>(&mut val, next);
            input.skip::<'{', '}'>(&mut val, next);
            input.skip::<'[', ']'>(&mut val, next);
            if into_key {
                input.skip_str_into_key(&mut key, next);
                if next == '"' {
                    into_key = false;
                    continue;
                }
            } else {
                input.skip_str(&mut val, next);
            }

            if next == ' ' {
                continue;
            }

            match next {
                ':' => {
                    into_key = false;
                    if input.peek() == Some(&' ') {
                        input.next();
                    }
                }
                // {
                '}' => {
                    into_key = true;
                    res.insert(
                        take(&mut key),
                        str_to_debug(ParsingStream::new(
                            &mut take(&mut val),
                        ))?,
                    );
                }
                ',' => {
                    into_key = true;
                    res.insert(
                        take(&mut key),
                        str_to_debug(ParsingStream::new(
                            &mut take(&mut val),
                        ))?,
                    );
                }
                _ => {
                    if into_key {
                        if next == ' ' {
                            if key.is_empty() {
                                continue;
                            } else {
                                return Err("Invalid key");
                            }
                        }
                        key.push(next);
                    } else {
                        val.push(next);
                    }
                }
            }
        }

        return Ok(DebugOutput::Map(res));
    }

    let mut prefix = String::new();
    let mut is_named = false;

    while let Some(next) = input.peek() {
        match next {
            // ' ' => {
            //     match input.peek() {
            //         Some('[' | '(' | '{') => {
            //             is_named = true;
            //             break;
            //         }
            //         // } ) ]
            //         _ => {}
            //     };
            // }
            '[' | '(' | '{' => {
                is_named = true;
                break;
            }
            // } ) ]
            _ => {
                let next = input.next().unwrap();
                prefix.push(next);
            }
        }
    }

    if is_named {
        if input.peek() == Some(&' ') {
            input.next();
        }

        let prefix = match prefix.chars().last() {
            Some(' ') => prefix.trim_end().to_string(),
            _ => prefix,
        };

        return Ok(DebugOutput::Named(
            prefix,
            Box::new(str_to_debug(input)?),
        ));
    } else {
        return Ok(DebugOutput::Other(prefix));
    }
}

#[track_caller]
pub fn value_to_be(

    expect: &DebugOutput,
    to_be: &DebugOutput,
    path: String,
) {
    match (expect, to_be) {
        
        (DebugOutput::Map(expect), DebugOutput::Map(to_be)) => {
            let mut keys = to_be.keys().collect::<Vec<_>>();
            for (key, value) in expect {
                if let Some(to_be) = to_be.get(key) {
                    keys.retain(|&x| x != key);
                    let path = format!("{}.{}", path, key);
                    value_to_be(value, to_be, path);
                } else {
                    let keys_missing = expect.keys().collect::<HashSet<_>>().difference(&to_be.keys().collect::<HashSet<_>>()).cloned().cloned().collect::<Vec<_>>();
                    panic!("{} has missing key(s) \n missing: ({})\nexpected: {:?}\n   found: {:?}", 
                        path, keys_missing.join(", "), expect, to_be
                    );
                }
            }

            for key in keys.iter() {
                panic!("{} some keys were not expected \n   extra: {:?}\nexpected: {:?}\n   found: {:?}"
                    ,path, 
                    keys,
                    expect, to_be
                )
            }
        }
        (DebugOutput::Vec(expect), DebugOutput::Vec(to_be)) => {
            if expect.len() == 1 && to_be.len() == 1 {
                // maybe it is a newtype
                return value_to_be(&expect[0], &to_be[0], path);
            }
            if expect.len() != to_be.len() {
                panic!(
                    "{} is an array with different lengths\n  length: expected {} but found to be {}\nexpected: {:?}\n   found: {:?}",
                    path, 
                    expect.len(),
                    to_be.len(),
                    expect, to_be
                );
            }


            for (index, expect) in expect.iter().enumerate() {
                let path = format!("{}[{}]", path, index);
                if let Some(to_be) = to_be.get(index) {
                    value_to_be(expect, to_be, path);
                } else {
                    unreachable!("{} keys did not match: \nexpected:{:?}\n  found:{:?}"
                        ,path, expect, to_be
                    )
                }
            }
        }
        (
            DebugOutput::Named(name, expect),
            DebugOutput::Named(name2, to_be),
        ) => {
            if name != name2 {
                panic!(
                    "{} type failure\n    type: expected {:?} found {:?}\nexpected: {:?}\n   found: {:?}",
                    path, name, name2, expect, to_be
                );
            }

            if expect != to_be {
                value_to_be(expect, to_be, path);
                       // format!("{}.{}", path, name));
            }
        }
        (
            DebugOutput::String(expect),
            DebugOutput::String(to_be),
        ) => {
            if expect != to_be {
                panic!(
                    "{} failed\nexpected: {:?}\n   found: {:?}",
                    path, expect, to_be
                );
            }
        }
        (
            DebugOutput::Other(expect),
            DebugOutput::Other(to_be),
        ) => {
            if expect != to_be {
                panic!(
                    "{} failed\nexpected: {:?}\n   found: {:?}",
                    path, expect, to_be
                );
            }
        }
        _ => {
            panic!(
                "{} failed\nexpected: {:?}\n   found: {:?}",
                path, expect, to_be
            );
        }
    }
}


fn pretty_comapre_fn(input: &str) -> String {
    let mut res = String::new();

    let mut peakable = input.chars().peekable();

    while let Some(c) = peakable.next() {
        if c.is_whitespace() {
            while let Some(next) = peakable.peek() {
                if next.is_whitespace() {
                    peakable.next();
                } else {
                    break;
                }
            }

            if let Some(next) = peakable.peek() {
                if next == &')' {
                    continue;
                }
            }

            res.push(' ');
            continue;
        }


        if c == '(' {
            while let Some(next) = peakable.peek() {
                if next.is_whitespace() {
                    peakable.next();
                } else {
                    break;
                }
            }
            res.push(c);
            continue;
        }

        res.push(c);
    }

    res
}

// this skips all whitespace between symbols "( { [ ] } ) : , ;"
// and letters, and between symbols themselves and converts all "
// into --[ and ]--
pub fn pretty_compare(expect: &str, tobe: &'static str) {
    let expect = pretty_comapre_fn(expect);
    let tobe = pretty_comapre_fn(tobe);

    assert_eq!(expect, tobe);
}

fn better_comapre_fn(input: &str) -> String {
    let mut res = String::new();

    let mut peakable = input.chars().peekable();

todo!();

    res
}

pub fn better_compare(expect: &str, tobe: &str) {
    let expect = better_comapre_fn(expect);
    let tobe = better_comapre_fn(tobe);

    assert_eq!(expect, tobe);
}
