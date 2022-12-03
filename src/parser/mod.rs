pub mod lexer;
pub mod location;

pub use lexer::LexicalError;
pub use location::Location;

use crate::ast;
use lalrpop_util::{lalrpop_mod, ParseError};
use lexer::Lexer;
use std::path::Path;
use std::{fs, io};

lalrpop_mod!(parser, "/parser/parser.rs");

pub fn parse<'input, S: Into<&'input Path>>(fname: S) -> Result<(), io::Error> {
    let path = fname.into();
    let raw = fs::read_to_string(path)?;

    println!("Start of toks in {:?}:\n===========", path.to_owned());
    for tok in Lexer::new(path.as_os_str().to_str().unwrap(), &raw) {
        let tok = tok.unwrap();
        println!("Read tok: {:?}", tok.1);
    }
    println!("===========\nEnd of toks in {:?}.", path.to_owned());

    let path = path.as_os_str().to_str().unwrap();
    let lexer = Lexer::new(path, &raw);
    let parser = parser::FileParser::new();
    match parser.parse(&raw, lexer) {
        Ok(ast) => println!(":D {:?}", ast),
        Err(err) => match err {
            ParseError::UnrecognizedEOF { .. } => println!("Unexpected EOF"),
            ParseError::UnrecognizedToken {
                token: (loc, tok, _), expected, ..
            } => println!("{}:{}:({}): unexpected token: expected one of {}, got {:?}", loc.file_name, loc.line, loc.index, expected.join(", "), tok),
            e => panic!("{:?}", e),
        },
    };

    Ok(())
}

struct File<'input> {
    name: String,
    root: ast::Node<'input>,
}
