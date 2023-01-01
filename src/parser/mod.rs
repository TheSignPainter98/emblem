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

    fn assert_structure(name: &str, input: &str, expected: &str) {
        // assert_eq!(
        //     expected,
        //     {
        //         let parse_result = parse(name, input);
        //         assert!(
        //             parse_result.is_ok(),
        //             "{}: expected Ok parse result when parsing {:?}, got: {:?}",
        //             name,
        //             input,
        //             parse_result
        //         );
        //         parse_result.unwrap().repr()
        //     },
        //     "{}",
        //     name
        // );
        let input_with_newline = &format!("{}\n", input);
        assert_eq!(
            expected,
            {
                let parse_result = parse(name, input_with_newline);
                assert!(
                    parse_result.is_ok(),
                    "{}: expected Ok parse result when parsing {:?}",
                    name,
                    input_with_newline
                );
                parse_result.unwrap().repr()
            },
            "{}",
            name
        );
    }

    mod paragraphs {
        use super::*;

        #[test]
        fn basic() {
            assert_eq!(parse_str("").unwrap().repr(), "File[]");
            // assert_eq!(parse_str("hello, world!").unwrap().repr(), "File[Par[Word(hello,)|Whitespace( )|Word(world!)]]");
            // assert_eq!(parse_str("hello, world!\n").unwrap().repr(), "File[Par[Word(hello,)|Whitespace( )|Word(world!)]]");
        }
    }

    mod multi_line_comments {
        use super::*;

        #[test]
        fn empty() {
            assert_structure("empty", "/**/", r"File[Par[[/*[]*/]]]");
            assert_structure(
                "multiple empty",
                "/**/\n\n/**/\n/**/",
                r"File[Par[[/*[]*/]]|Par[[/*[]*/]|[/*[]*/]]]",
            );

        }

        #[test]
        fn whitespace_contents() {
            assert_structure("space only", "/* */", r"File[Par[[/*[ ]*/]]]");
            assert_structure("tab only", "/*\t*/", r"File[Par[[/*[\t]*/]]]");
        }

        #[test]
        fn with_text() {
            assert_structure(
                "text",
                "/*spaghetti and meatballs*/",
                r"File[Par[[/*[spaghetti and meatballs]*/]]]",
            );
            assert_structure(
                "text with surrounding space",
                "/* spaghetti and meatballs */",
                r"File[Par[[/*[ spaghetti and meatballs ]*/]]]",
            );
            assert_structure(
                "text with newline",
                "/*spaghetti and\nmeatballs*/",
                r"File[Par[[/*[spaghetti and|\n|meatballs]*/]]]",
            );
            assert_structure(
                "multiple comments",
                "/*spaghetti*/\n/*and*/\n\n/*meatballs*/",
                r"File[Par[[/*[spaghetti]*/]|[/*[and]*/]]|Par[[/*[meatballs]*/]]]",
            );
        }

        #[test]
        fn nested() {
            assert_structure(
                "nested comment",
                "/*spaghetti/*and*/meatballs*/",
                r"File[Par[[/*[spaghetti|Nested[and]|meatballs]*/]]]",
            );
            assert_structure(
                "nested and indented comment",
                "/*spaghetti\n\t/*\n\t\tand\n\t*/\nmeatballs*/",
                r"File[Par[[/*[spaghetti|\n|\t|Nested[\n|\t\tand|\n|\t]|\n|meatballs]*/]]]",
            );
            assert_structure(
                "nested unindented comment",
                "/*spaghetti\n\t/*\nand\n\t*/\nmeatballs*/",
                r"File[Par[[/*[spaghetti|\n|\t|Nested[\n|and|\n|\t]|\n|meatballs]*/]]]",
            );
        }

        #[test]
        fn unmatched_close() {
            assert!(parse_str("/*spaghetti/*and*/meatballs").is_err());
        }

        #[test]
        fn final_indentation() {
            assert!(parse_str("/*spaghetti\n\t*/").is_err());
            assert!(parse_str("/*spaghetti\n    */").is_err());
            assert_structure(
                "long, prettified comment block",
                "/* spaghetti\n *and\n *meatballs\n */",
                r"File[Par[[/*[ spaghetti|\n| *and|\n| *meatballs|\n| ]*/]]]",
            );
            // TODO(kcza): test bad leaving-indentation with trailing args
        }
    }
}
