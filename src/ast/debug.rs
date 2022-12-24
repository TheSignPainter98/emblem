#![cfg(test)]
use crate::ast::{File, Par};
use lazy_static::lazy_static;
use regex::Regex;

#[cfg(test)]
pub trait AstDebug {
    fn fmt(&self, buf: &mut Vec<String>);

    fn repr(&self) -> String {
        let mut buf = vec![];
        self.fmt(&mut buf);
        buf.join("")
    }
}

impl AstDebug for &str {
    fn fmt(&self, buf: &mut Vec<String>) {
        lazy_static! {
            static ref ILLEGAL: Regex = Regex::new(r"([|()\[\]])").unwrap();
        };
        buf.push(ILLEGAL.replace_all(&self, r"\$1").to_string());
    }
}

impl AstDebug for String {
    fn fmt(&self, buf: &mut Vec<String>) {
        self.as_str().fmt(buf);
    }
}

impl<T: AstDebug> AstDebug for Vec<T> {
    fn fmt(&self, buf: &mut Vec<String>) {
        buf.push("[".into());
        for (i, v) in self.iter().enumerate() {
            if i > 0 {
                buf.push("|".into())
            }
            v.fmt(buf)
        }
        buf.push("]".into());
    }
}

impl<T: AstDebug> AstDebug for Box<T> {
    fn fmt(&self, buf: &mut Vec<String>) {
        self.as_ref().fmt(buf);
    }
}

mod test {
    use super::*;

    fn test_str_like<T:AstDebug>(f: fn(&'static str) -> T) {
        assert_eq!(r"", AstDebug::repr(&f("")));
        assert_eq!(r"hello, world", AstDebug::repr(&f("hello, world")));
        assert_eq!(r"\|", AstDebug::repr(&f("|")));
        assert_eq!(r"\(", AstDebug::repr(&f("(")));
        assert_eq!(r"\)", AstDebug::repr(&f(")")));
        assert_eq!(r"\]", AstDebug::repr(&f("]")));
        assert_eq!(r"\[", AstDebug::repr(&f("[")));
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
    fn r#box() {
        test_str_like(|s| Box::new(s));
    }

    #[test]
    fn vec() {
        assert_eq!(r"[]", AstDebug::repr(&Vec::<&str>::new()));
        assert_eq!(r"[hello|world]", AstDebug::repr(&vec!["hello", "world"]));
        assert_eq!(r"[\[|\]|\(|\)|\|]", AstDebug::repr(&vec!["[", "]", "(", ")", "|"]));
    }
}
