pub mod lexer;
pub mod location;

pub use lexer::LexicalError;
pub use location::Location;

use crate::ast;
use lalrpop_util::{lalrpop_mod, ParseError};
use lexer::Lexer;
use std::path::Path;
use std::{fs, io};
use ast::region::Region;

lalrpop_mod!(parser, "/parser/parser.rs");

pub fn parse<'input, S: Into<&'input Path>>(fname: S) -> Result<(), io::Error> {
    let path = fname.into();
    let raw = fs::read_to_string(path)?;

    println!("Start of toks in {:?}:\n===========", path.to_owned());
    for tok in Lexer::new(path.as_os_str().to_str().unwrap(), &raw) {
        match tok {
            Ok(tok) => println!("Read tok {}: {:?}", Region::new(&tok.0, &tok.2), tok.1),
            Err(ref err) => println!("Lexical error: {:?}", err),
        }
    }
    println!("===========\nEnd of toks in {:?}.", path.to_owned());

    let path = path.as_os_str().to_str().unwrap();
    let lexer = Lexer::new(path, &raw);
    let parser = parser::FileParser::new();
    match parser.parse(&raw, lexer) {
        Ok(ast) => println!(":D {:?}", ast),
        Err(err) => match err {
            ParseError::UnrecognizedEOF { location, expected } => println!(
                "{}: Unexpected EOF, expected one of {}",
                location,
                pretty_tok_list(expected),
            ),
            ParseError::UnrecognizedToken {
                token: (loc, tok, _),
                expected,
            } => println!(
                "{}: expected {}, before {:?} token",
                loc,
                pretty_tok_list(expected),
                tok
            ),
            ParseError::User { error: err } => println!("{}: {}", err.location(), err),
            ParseError::InvalidToken { location } => println!("{}: invalid token", location),
            ParseError::ExtraToken {
                token: (loc, tok, _),
            } => println!("{}: unexpected extra token: {}", loc, tok.to_string()),
        },
    };

    Ok(())
}

fn pretty_tok_list(list: Vec<String>) -> String {
    let len = list.len();
    let mut pretty_list = Vec::new();
    for (i, e) in list.iter().enumerate() {
        if i > 0 {
            pretty_list.push(if i < len-1 { ", " } else { " or " })
        }
        pretty_list.push(e);
    }
    pretty_list.concat()
}

struct File<'input> {
    name: String,
    root: ast::Node<'input>,
}
