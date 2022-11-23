pub mod lexer;

use crate::ast;
use lexer::Lexer;
use std::error::Error;
use std::path::Path;
use std::{
    fmt::{self, Display},
    fs, io,
};

pub fn parse<'input, S: Into<&'input Path>>(fname: S) -> Result<(), io::Error> {
    let path = fname.into();
    let raw = fs::read_to_string(path)?;

    println!("Start of toks in {:?}:\n===========", path.to_owned());
    for tok in Lexer::new(path.as_os_str().to_str().unwrap(), &raw) {
        println!("Read tok: {:?}", tok);
    }
    println!("===========\nEnd of toks in {:?}:", path.to_owned());

    Ok(())
}

struct File<'input> {
    name: String,
    root: ast::Node<'input>,
}

#[derive(Clone, Debug)]
pub struct Location<'input> {
    pub text: &'input str,
    pub file: &'input str,
    pub line: usize,
    pub col: usize,
}

impl<'input> Location<'input> {
    fn new(file: &'input str) -> Self {
        Self {
            text: "",
            file,
            line: 1,
            col: 0,
        }
    }

    fn with_text(&self, text: &'input str) -> Self {
        let mut ret = self.clone();
        ret.text = text;
        ret
    }

    fn incr_line(&mut self) {
        self.col = 1;
        self.line += 1;
    }

    fn incr_col(&mut self) {
        self.col += 1;
    }
}

impl Display for Location<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}:", self.file, self.line, self.col)
    }
}

#[cfg(test)]
mod test {
    mod location {
        use super::super::*;
        #[test]
        fn new() {
            assert_eq!("asdf", Location::new("asdf").file);
        }

        #[test]
        fn with_text() {
            let original = Location::new("fname");
            let new = original.with_text("hello, world");

            assert_eq!(new.file, original.file);
            assert_eq!(new.text, "hello, world");
            assert_eq!(new.col, original.col);
            assert_eq!(new.line, original.line);
        }

        #[test]
        fn incr() {
            let mut loc = Location::new("");

            loc.incr_col();
            assert_eq!((loc.line, loc.col), (1, 1));

            for _ in 0..10 {
                loc.incr_col();
            }

            assert_eq!((loc.line, loc.col), (1, 11));

            loc.incr_line();

            assert_eq!((loc.line, loc.col), (2, 1));

            for _ in 0..10 {
                loc.incr_line()
            }

            assert_eq!((loc.line, loc.col), (12, 1));

            for _ in 0..10 {
                loc.incr_col();
            }

            assert_eq!((loc.line, loc.col), (12, 11));
        }
    }
}
