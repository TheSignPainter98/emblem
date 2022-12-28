#![cfg(test)]
use crate::ast::{File, Par};
use lazy_static::lazy_static;
use regex::Regex;

#[cfg(test)]
pub trait AstDebug {
    fn test_fmt(&self, buf: &mut Vec<String>);

    fn repr(&self) -> String {
        let mut buf = vec![];
        self.test_fmt(&mut buf);
        buf.join("")
    }

    fn surround<S, T>(&self, buf: &mut Vec<String>, before: S, after: T)
    where
        S: Into<String>,
        T: Into<String>,
    {
        buf.push(before.into());
        self.test_fmt(buf);
        buf.push(after.into());
    }
}

impl AstDebug for &str {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        lazy_static! {
            static ref ILLEGAL: Regex = Regex::new(r"([|()\[\]])").unwrap();
        };

        let sanitised = self;
        let sanitised = format!("{:?}", sanitised);
        let sanitised = ILLEGAL.replace_all(&sanitised, r"\$1").to_string();
        let sanitised = &sanitised[1..sanitised.len() - 1];

        buf.push(sanitised.to_owned());
    }
}

impl AstDebug for String {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        self.as_str().test_fmt(buf);
    }
}

impl<T: AstDebug> AstDebug for Vec<T> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        buf.push("[".into());
        for (i, v) in self.iter().enumerate() {
            if i > 0 {
                buf.push("|".into())
            }
            v.test_fmt(buf)
        }
        buf.push("]".into());
    }
}

impl<T: AstDebug> AstDebug for Box<T> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        self.as_ref().test_fmt(buf);
    }
}

mod test {
    use super::*;

    fn test_str_like<T: AstDebug>(f: fn(&'static str) -> T) {
        assert_eq!(r"", f("").repr());
        assert_eq!(r"hello, world", f("hello, world").repr());
        assert_eq!(r"\t", f("\t").repr());
        assert_eq!(r"\\", f(r"\").repr());
        assert_eq!(r"\\\|", f(r"\|").repr());
        assert_eq!(r"\|", f("|").repr());
        assert_eq!(r"\(", f("(").repr());
        assert_eq!(r"\)", f(")").repr());
        assert_eq!(r"\]", f("]").repr());
        assert_eq!(r"\[", f("[").repr());
        assert_eq!(r#"\""#, f("\"").repr());
        assert_eq!(r"'", f("'").repr());
    }

    #[test]
    fn str() {
        test_str_like(|s| s);
    }

    #[test]
    fn string() {
        test_str_like(|s| s.to_string())
    }

    #[test]
    fn boxed() {
        test_str_like(|s| Box::new(s));
    }

    #[test]
    fn vec() {
        assert_eq!(r"[]", &Vec::<&str>::new().repr());
        assert_eq!(r"[hello|world]", vec!["hello", "world"].repr());
        assert_eq!(r"[hello|world]", vec!["hello", "world"].repr());
        assert_eq!(r"[\[|\]|\(|\)|\|]", vec!["[", "]", "(", ")", "|"].repr());
    }
}
