pub mod lexer;

use crate::ast;
use lazy_static::lazy_static;
use lexer::Lexer;
use regex::Regex;
use std::error::Error;
use std::path::Path;
use std::{
    fmt::{self, Display},
    fs, io,
};

lazy_static! {
    static ref NEWLINE: Regex = Regex::new("(\n|\r\n|\r)").unwrap();
}

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

#[derive(Copy, Clone, Debug)]
pub struct Location<'input> {
    pub file_name: &'input str,
    src: &'input str,
    offset: usize,
    pub line: usize,
    pub col: usize,
}

impl<'input> Location<'input> {
    fn new(fname: &'input str, src: &'input str) -> Self {
        Self {
            file_name: fname,
            src,
            offset: 0,
            line: 1,
            col: 1,
        }
    }

    fn shift(mut self, text: &'input str) -> Self {
        self.offset += text.len();

        let mut line_count = 0;
        let mut last_line = None;

        for line in NEWLINE.split(text) {
            last_line = Some(line);
            line_count += 1;
        }

        self.line += line_count - 1;

        let last_line_chars = last_line.unwrap().chars().count();
        if line_count > 1 {
            self.col = 1 + last_line_chars;
        } else {
            self.col += last_line_chars;
        }
        self
    }

    pub fn text_upto(&self, other: &Location) -> &'input str {
        &self.src[self.offset..other.offset]
    }
}

impl Display for Location<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file_name, self.line, self.col)
    }
}

#[cfg(test)]
mod test {
    mod location {
        use super::super::*;
        #[test]
        fn new() {
            let loc = Location::new("fname", "content");
            assert_eq!("fname", loc.file_name);
            assert_eq!(1, loc.col);
            assert_eq!(1, loc.line);
        }

        #[test]
        fn shift_single_line() {
            let start = Location::new("fname", "my name is methos");
            let mid = start.shift("my name is ");
            let end = mid.shift("methos");

            assert_eq!(mid.file_name, "fname");
            assert_eq!(mid.col, 12);
            assert_eq!(mid.line, 1);

            assert_eq!(start.file_name, end.file_name);
            assert_eq!(18, end.col);
            assert_eq!(1, end.line);

            assert_eq!("my name is ", start.text_upto(&mid));
            assert_eq!("methos", mid.text_upto(&end));
            assert_eq!("my name is methos", start.text_upto(&end));
        }

        #[test]
        fn shift_multi_line() {
            let src = "Welcome! Welcome to City 17! You have chosen, or been chosen, to relocate to one of our finest remaining urban centres";
            let lines = src.replace(" ", "\n");
            let start = Location::new("fname", &lines);
            let end = start.clone().shift(&lines);

            assert_eq!(21, end.line);
            assert_eq!(8, end.col);
            assert_eq!(lines, start.text_upto(&end));
        }
    }
}
