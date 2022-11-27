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
        println!("Read tok: {}", tok.unwrap().0);
    }
    println!("===========\nEnd of toks in {:?}:", path.to_owned());

    Ok(())
}

struct File<'input> {
    name: String,
    root: ast::Node<'input>,
}
