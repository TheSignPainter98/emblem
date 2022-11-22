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

#[derive(Debug)]
pub struct Location<'input> {
    text: &'input str,
    file: &'input str,
    line: usize,
    col: usize,
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
}

impl Display for Location<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}:", self.file, self.line, self.col)
    }
}
