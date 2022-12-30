pub mod lexer;
pub mod location;

pub use lexer::LexicalError;
pub use location::Location;

use crate::ast;
use lalrpop_util::lalrpop_mod;
// use lalrpop_util::ParseError;
use crate::context::Context;
use ast::ParsedAst;
use lexer::Lexer;
use std::error::Error;
use std::fs;

lalrpop_mod!(
    #[allow(clippy::all)]
    parser,
    "/parser/parser.rs"
);

/// Parse an emblem source file at the given location.
pub fn parse_file<'ctx>(
    ctx: &'ctx mut Context,
    path: String,
) -> Result<ParsedAst<'ctx>, Box<dyn Error + 'ctx>>
{
    let content = fs::read_to_string(&path)?;
    let file = ctx.alloc_file(path, content);

    parse(file.name(), file.content())
}

pub fn parse<'file>(
    fname: &'file str,
    input: &'file str,
) -> Result<ParsedAst<'file>, Box<dyn Error + 'file>> {
    let lexer = Lexer::new(fname, input);
    let parser = parser::FileParser::new();

    Ok(parser.parse(lexer)?)
}

/// Create a string representation of a list of tokens which will fit in with surrounding text.
fn pretty_tok_list(list: Vec<String>) -> String {
    let len = list.len();
    let mut pretty_list = Vec::new();
    for (i, e) in list.iter().enumerate() {
        if i > 0 {
            pretty_list.push(if i < len - 1 { ", " } else { " or " })
        }
        pretty_list.push(e);
    }
    pretty_list.concat()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::{parsed::Content, AstDebug, File};

    fn parse_str<'i>(input: &'i str) -> Result<ParsedAst<'i>, Box<dyn Error + 'i>> {
        parse("test.em", input)
    }

    #[test]
    fn basic() {
        assert_eq!(parse_str("").unwrap().repr(), "File[]");
    }

    #[test]
    fn multi_line_comments() {
        assert_eq!(
            parse_str("/**/").unwrap().repr(),
            r"File[Par[/*[]*/]]"
        );
        assert_eq!(
            parse_str("/* */").unwrap().repr(),
            r"File[Par[/*[ ]*/]]"
        );
        assert_eq!(
            parse_str("/*\t*/").unwrap().repr(),
            r"File[Par[/*[\t]*/]]"
        );
        assert_eq!(
            parse_str("/*spaghetti and meatballs*/").unwrap().repr(),
            r"File[Par[/*[spaghetti and meatballs]*/]]"
        );
        assert_eq!(
            parse_str("/* spaghetti and meatballs */").unwrap().repr(),
            r"File[Par[/*[ spaghetti and meatballs ]*/]]"
        );
        assert_eq!(
            parse_str("/*spaghetti and\nmeatballs*/").unwrap().repr(),
            r"File[Par[/*[spaghetti and|\n|meatballs]*/]]"
        );
        assert_eq!(
            parse_str("/*spaghetti*/\n/*and*/\n\n/*meatballs*/")
                .unwrap()
                .repr(),
            r"File[Par[/*[spaghetti]*/|/*[and]*/]|Par[/*[meatballs]*/]]"
        );
        assert_eq!(
            parse_str("/*spaghetti\n\tand\nmeatballs*/").unwrap().repr(),
            r"File[Par[/*[spaghetti|Indented[and]|meatballs]*/]]"
        );
        assert_eq!(
            parse_str("/*spaghetti/*and*/meatballs*/").unwrap().repr(),
            r"File[Par[/*[spaghetti|Nested[and]|meatballs]*/]]"
        );
        assert_eq!(
            parse_str("/*spaghetti\n\t/*\n\t\tand\n\t*/\nmeatballs*/").unwrap().repr(),
            r"File[Par[/*[spaghetti|Nested[and]|meatballs]*/]]"
        );
    }
}
