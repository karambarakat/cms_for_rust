#![allow(deprecated)]

#[deprecated]
pub struct UnPrettyStr(String);

impl UnPrettyStr {
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for UnPrettyStr {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl ToString for UnPrettyStr {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl From<&str> for UnPrettyStr {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

#[deprecated]
pub trait UnPretty {
    fn un_pretty(self) -> UnPrettyStr;
}

pub(crate) fn skip_white_space<I>(iter: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = char>,
{
    while let Some(next) = iter.peek() {
        if next.is_whitespace() {
            iter.next();
        } else {
            break;
        }
    }
}

impl UnPretty for &str {
    fn un_pretty(self) -> UnPrettyStr {
        let mut result = String::new();

        let mut peek = self.chars().peekable();

        while let Some(c) = peek.next() {
            if c.is_whitespace() {
                skip_white_space(&mut peek);

                if let Some(next) = peek.peek() {
                    if next == &')' {
                        continue;
                    }
                }

                result.push(' ');
                continue;
            }

            // if c == '`' {
            //     result.push('"');
            //     continue;
            // }

            if c == '(' {
                skip_white_space(&mut peek);
                result.push(c);
                continue;
            }

            result.push(c);
        }

        UnPrettyStr(result.trim().into())
    }
}
