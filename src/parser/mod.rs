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
        assert_eq!(
            {
                let parse_result = parse(name, input);
                assert!(
                    parse_result.is_ok(),
                    "{}: expected Ok parse result when parsing {:?}, got: {:?}",
                    name,
                    input,
                    parse_result.unwrap_err(),
                );
                parse_result.unwrap().repr()
            },
            expected,
            "{}",
            name
        );

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

    fn assert_parse_error(name: &str, input: &str) {
        // assert!(parse(name, input).is_err(), "{}", name);

        let input_with_newline = &format!("{}\n", input);
        assert!(parse(name, input_with_newline).is_err(), "{}", name);
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
                "Spiderpig, Spiderpig,\n\nDoes whatever a Spiderpig does.\n\nCan he swing from a web?\n\nNo, he can't, he's a pig,\n\nLook out, he is a Spiderpig!",
                "File[Par[[Word(Spiderpig,)|< >|Word(Spiderpig,)]]|Par[[Word(Does)|< >|Word(whatever)|< >|Word(a)|< >|Word(Spiderpig)|< >|Word(does.)]]|Par[[Word(Can)|< >|Word(he)|< >|Word(swing)|< >|Word(from)|< >|Word(a)|< >|Word(web?)]]|Par[[Word(No,)|< >|Word(he)|< >|Word(can't,)|< >|Word(he's)|< >|Word(a)|< >|Word(pig,)]]|Par[[Word(Look)|< >|Word(out,)|< >|Word(he)|< >|Word(is)|< >|Word(a)|< >|Word(Spiderpig!)]]]",
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

    mod commands {
        use super::*;

        #[test]
        fn command_only() {
            assert_structure("command", ".order66", "File[Par[[.order66]]]")
        }

        #[test]
        fn with_args() {
            assert_structure(
                "sole",
                ".exec{order66}",
                "File[Par[[.exec{[Word(order66)]}]]]",
            );
            assert_structure("start of line", ".old-man-say{leave her Johnny, leave her} tomorrow ye will get your pay", "File[Par[[.old-man-say{[Word(leave)|< >|Word(her)|< >|Word(Johnny,)|< >|Word(leave)|< >|Word(her)]}|< >|Word(tomorrow)|< >|Word(ye)|< >|Word(will)|< >|Word(get)|< >|Word(your)|< >|Word(pay)]]]");
            assert_structure("end of line", "I hate to .sail{on this rotten tub}", "File[Par[[Word(I)|< >|Word(hate)|< >|Word(to)|< >|.sail{[Word(on)|< >|Word(this)|< >|Word(rotten)|< >|Word(tub)]}]]]");
            assert_structure("middle of line", "For the .voyage-is{foul} and the winds don't blow", "File[Par[[Word(For)|< >|Word(the)|< >|.voyage-is{[Word(foul)]}|< >|Word(and)|< >|Word(the)|< >|Word(winds)|< >|Word(don't)|< >|Word(blow)]]]");
            assert_structure("nested", ".no{grog .allowed{and} rotten grub}", "File[Par[[.no{[Word(grog)|< >|.allowed{[Word(and)]}|< >|Word(rotten)|< >|Word(grub)]}]]]");

            assert_parse_error("orphaned open brace", "{");
            assert_parse_error("orphaned close brace", "}");
            assert_parse_error("superfluous open brace", ".order66{}{");
            assert_parse_error("superfluous close brace", ".order66{}}");

            assert_parse_error("newline in brace-arg", ".order66{\n}");
            assert_parse_error("newline in brace-arg", ".order66{general\nkenobi}");
            assert_parse_error("par-break in brace-arg", ".order66{\n\n}");
            assert_parse_error("par-break in brace-arg", ".order66{general\n\nkenobi}");
        }

        #[test]
        fn remainder_args() {
            assert_structure("start of line", ".now{we are ready}: to sail for the horn", "File[Par[[.now{[Word(we)|< >|Word(are)|< >|Word(ready)]}:[Word(to)|< >|Word(sail)|< >|Word(for)|< >|Word(the)|< >|Word(horn)]]]]");
            assert_structure(
                "middle of line",
                "our boots .and{our clothes boys}, are all in the pawn",
                "File[Par[[Word(our)|< >|Word(boots)|< >|.and{[Word(our)|< >|Word(clothes)|< >|Word(boys)]}|Word(,)|< >|Word(are)|< >|Word(all)|< >|Word(in)|< >|Word(the)|< >|Word(pawn)]]]",
            );
        }
    }

    mod interword {
        use super::*;

        struct InterwordTest {
            input: String,
            expected: String,
        }

        fn test_interword(name: &str, dash: &str, repr: &str) {
            let inputs = vec![
                InterwordTest {
                    input: dash.into(),
                    expected: format!("File[Par[[{}]]]", repr),
                },
                InterwordTest {
                    input: format!("a{}b", dash),
                    expected: format!("File[Par[[Word(a)|{}|Word(b)]]]", repr),
                },
                InterwordTest {
                    input: format!("a {}b", dash),
                    expected: format!("File[Par[[Word(a)|< >|{}|Word(b)]]]", repr),
                },
                InterwordTest {
                    input: format!("a{} b", dash),
                    expected: format!("File[Par[[Word(a)|{}|< >|Word(b)]]]", repr),
                },
                InterwordTest {
                    input: format!("a {} b", dash),
                    expected: format!("File[Par[[Word(a)|< >|{}|< >|Word(b)]]]", repr),
                },
                InterwordTest {
                    input: format!("a\n{}b", dash),
                    expected: format!("File[Par[[Word(a)]|[{}|Word(b)]]]", repr),
                },
                InterwordTest {
                    input: format!("a{}\nb", dash),
                    expected: format!("File[Par[[Word(a)|{}]|[Word(b)]]]", repr),
                },
            ];
            for InterwordTest { input, expected } in inputs {
                assert_structure(name, &input, &expected);
            }
        }

        #[test]
        fn hyphen() {
            test_interword("hyphen", "-", "-");
        }

        #[test]
        fn en() {
            test_interword("en", "--", "--");
        }

        #[test]
        fn em() {
            test_interword("em", "---", "---");
        }

        #[test]
        fn glue() {
            test_interword("em", "~", "~");
        }

        #[test]
        fn nbsp() {
            test_interword("em", "~~", "~~");
        }

        #[test]
        fn mixed() {
            test_interword("em-hyph", "----", "---|-");
            test_interword("em-en", "-----", "---|--");
            test_interword("em-em", "------", "---|---");
            test_interword("glue-mixed-1-dash-1", "~-", "~|-");
            test_interword("glue-mixed-1-dash-2", "-~", "-|~");
            test_interword("glue-mixed-2-dashes-1", "~--", "~|--");
            test_interword("glue-mixed-2-dashes-2", "-~-", "-|~|-");
            test_interword("glue-mixed-2-dashes-3", "--~", "--|~");
            test_interword("glue-mixed-3-dashes-1", "~---", "~|---");
            test_interword("glue-mixed-3-dashes-2", "-~--", "-|~|--");
            test_interword("glue-mixed-3-dashes-3", "--~-", "--|~|-");
            test_interword("glue-mixed-3-dashes-4", "---~", "---|~");
            test_interword("glue-mixed-1-dashes-1", "~~-", "~~|-");
            test_interword("glue-mixed-1-dashes-2", "-~~", "-|~~");
            test_interword("glue-mixed-2-dashes-1", "~~--", "~~|--");
            test_interword("glue-mixed-2-dashes-2", "-~~-", "-|~~|-");
            test_interword("glue-mixed-2-dashes-3", "--~~", "--|~~");
            test_interword("glue-mixed-3-dashes-1", "~~---", "~~|---");
            test_interword("glue-mixed-3-dashes-2", "-~~--", "-|~~|--");
            test_interword("glue-mixed-3-dashes-3", "--~~-", "--|~~|-");
            test_interword("glue-mixed-3-dashes-4", "---~~", "---|~~");
        }
    }

    mod verbatim {
        use super::*;

        #[test]
        fn word() {
            assert_structure(
                "ignore unmatched at start",
                "spanish inquisition!",
                "File[Par[[Word(spanish)|< >|Word(inquisition!)]]]",
            );
            assert_structure(
                "ignore unmatched at end",
                "!spanish inquisition",
                "File[Par[[Word(!spanish)|< >|Word(inquisition)]]]",
            );
        }

        #[test]
        fn short() {
            assert_structure("text", "!verb!", "File[Par[[!verb!]]]");
            assert_structure("comment", "!//!", "File[Par[[!//!]]]");
            assert_structure("multi line comment start", "!/*!", "File[Par[[!/*!]]]");
            assert_structure("multi line comment end", "!*/!", "File[Par[[!*/!]]]");
            assert_structure("empty", "!!", "File[Par[[!!]]]");
            assert_structure(
                "with spaces",
                "!hello } world :: !",
                "File[Par[[!hello } world :: !]]]",
            );
            assert_structure("ignored in comment", "//!asdf!", "File[Par[[//!asdf!]]]");
        }
    }

    mod single_line_comments {
        use super::*;

        #[test]
        fn whole_line() {
            assert_structure(
                "no gap",
                "//hello, world!\n",
                "File[Par[[//hello, world!]]]",
            );
            assert_structure(
                "leading space",
                "// hello, world!\n",
                "File[Par[[// hello, world!]]]",
            );
            assert_structure(
                "leading tab",
                "//\thello, world!\n",
                r"File[Par[[//\thello, world!]]]",
            );
        }

        #[test]
        fn partial() {
            assert_structure(
                "whole_line",
                "to me!//to you!\n",
                "File[Par[[Word(to)|< >|Word(me!)|//to you!]]]",
            );
            assert_structure(
                "whole_line",
                "to me!// to you!\n",
                "File[Par[[Word(to)|< >|Word(me!)|// to you!]]]",
            );
            assert_structure(
                "whole_line",
                "to me! //to you!\n",
                "File[Par[[Word(to)|< >|Word(me!)|< >|//to you!]]]",
            );
            assert_structure(
                "whole_line",
                "to me! // to you!\n",
                "File[Par[[Word(to)|< >|Word(me!)|< >|// to you!]]]",
            );
        }

        #[test]
        fn stacked() {
            let lines = vec![
                "There once was a ship that put to sea",
                "And the name of that ship was the Billy O’ Tea",
                "The winds blew hard, her bow dipped down",
                "Blow, me bully boys, blow",
            ];
            assert_structure(
                "whole_line",
                &format!("//{}\n", lines.join("\n//")),
                &format!(
                    "File[Par[[{}]]]",
                    lines
                        .iter()
                        .map(|l| format!("//{}", l))
                        .collect::<Vec<_>>()
                        .join("]|[")
                ),
            );
        }

        #[test]
        fn as_trailing_arg() {
            assert_structure(
                "comment as sole arg",
                ".spaghetti:\n\t//and meatballs",
                "File[Par[[.spaghetti::[Par[[//and meatballs]]]]]]",
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
        fn as_trailing_arg() {
            assert_structure(
                "comment as sole arg",
                ".spaghetti:\n\t/*and meatballs*/",
                "File[Par[[.spaghetti::[Par[[/*[and meatballs]*/]]]]]]",
            );
        }

        #[test]
        fn final_indentation() {
            assert_structure(
                "final tab indent",
                "/*spaghetti\n\t*/",
                r"File[Par[[/*[spaghetti|\n|\t]*/]]]",
            );
            assert_structure(
                "final spaces indent",
                "/*spaghetti\n    */",
                r"File[Par[[/*[spaghetti|\n|    ]*/]]]",
            );
            assert_structure(
                "long, prettified comment block",
                "/* spaghetti\n *and\n *meatballs\n */",
                r"File[Par[[/*[ spaghetti|\n| *and|\n| *meatballs|\n| ]*/]]]",
            );
            // TODO(kcza): test bad leaving-indentation with trailing args
        }
    }
}
