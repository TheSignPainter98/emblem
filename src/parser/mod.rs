pub mod lexer;
pub mod location;

pub use crate::parser::location::Location;

use crate::ast;
use lexer::Lexer;
use std::path::Path;
use std::{fs, io};

pub fn parse<'input, S: Into<&'input Path>>(fname: S) -> Result<(), io::Error> {
    let path = fname.into();
    let raw = fs::read_to_string(path)?;

    println!("Start of toks in {:?}:\n===========", path.to_owned());
    for tok in Lexer::new(path.as_os_str().to_str().unwrap(), &raw) {
        let tok = tok.unwrap();
        println!("Read tok: {:?}", tok.1);
    }
    println!("===========\nEnd of toks in {:?}.", path.to_owned());

    Ok(())
}

struct File<'input> {
    name: String,
    root: ast::Node<'input>,
}
