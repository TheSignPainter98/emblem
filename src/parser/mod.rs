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
        fn empty() {
            assert_structure("empty", "", "File[Par[[]]]");
        }

        #[test]
        fn single_line() {
            assert_structure(
                "single line",
                "hello, world!",
                "File[Par[[Word(hello,)|< >|Word(world!)]]]",
            );

            assert_structure(
                "single line with tabs",
                "hello,\tworld!",
                r"File[Par[[Word(hello,)|<\t>|Word(world!)]]]",
            );

            assert_structure(
                "single line for many pars",
                "Spider-Pig, Spider-Pig,\n\nDoes whatever a Spider-Pig does.\n\nCan he swing from a web?\n\nNo, he can't, he's a pig,\n\nLook out, he is a Spider-Pig!",
                "File[Par[[Word(Spider-Pig,)|< >|Word(Spider-Pig,)]]|Par[[Word(Does)|< >|Word(whatever)|< >|Word(a)|< >|Word(Spider-Pig)|< >|Word(does.)]]|Par[[Word(Can)|< >|Word(he)|< >|Word(swing)|< >|Word(from)|< >|Word(a)|< >|Word(web?)]]|Par[[Word(No,)|< >|Word(he)|< >|Word(can't,)|< >|Word(he's)|< >|Word(a)|< >|Word(pig,)]]|Par[[Word(Look)|< >|Word(out,)|< >|Word(he)|< >|Word(is)|< >|Word(a)|< >|Word(Spider-Pig!)]]]",
            );
        }

        #[test]
        fn multiple_lines() {
            assert_structure("multiple lines",
                "According to all known laws of aviation, there is no way that a bee should be able to fly.\nIts wings are too small to get its fat little body off the ground.\n\nThe bee, of course, flies anyway because bees don't care what humans think is impossible.",
                "File[Par[[Word(According)|< >|Word(to)|< >|Word(all)|< >|Word(known)|< >|Word(laws)|< >|Word(of)|< >|Word(aviation,)|< >|Word(there)|< >|Word(is)|< >|Word(no)|< >|Word(way)|< >|Word(that)|< >|Word(a)|< >|Word(bee)|< >|Word(should)|< >|Word(be)|< >|Word(able)|< >|Word(to)|< >|Word(fly.)]|[Word(Its)|< >|Word(wings)|< >|Word(are)|< >|Word(too)|< >|Word(small)|< >|Word(to)|< >|Word(get)|< >|Word(its)|< >|Word(fat)|< >|Word(little)|< >|Word(body)|< >|Word(off)|< >|Word(the)|< >|Word(ground.)]]|Par[[Word(The)|< >|Word(bee,)|< >|Word(of)|< >|Word(course,)|< >|Word(flies)|< >|Word(anyway)|< >|Word(because)|< >|Word(bees)|< >|Word(don't)|< >|Word(care)|< >|Word(what)|< >|Word(humans)|< >|Word(think)|< >|Word(is)|< >|Word(impossible.)]]]",
            );
        }
    }

    mod multi_line_comments {
        use super::*;

        #[test]
        fn empty() {
            assert_structure("empty", "/**/", r"File[Par[[/*[]*/]]]");
            assert_structure("empty", "/**//**/", r"File[Par[[/*[]*/|/*[]*/]]]");
            assert_structure("empty", "/**/ /**/", r"File[Par[[/*[]*/|< >|/*[]*/]]]");
            assert_structure("empty", "/**/\t/**/", r"File[Par[[/*[]*/|<\t>|/*[]*/]]]");
            assert_structure("empty", "/**/\n/**/", r"File[Par[[/*[]*/]|[/*[]*/]]]");
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
