mod error;
pub mod lexer;
pub mod location;
mod point;

pub use error::Error;
pub use lexer::LexicalError;
pub use location::Location;

use crate::args::SearchResult;
use crate::ast;
use crate::context::Context;
use ast::parsed::ParsedFile;
use error::OsStringConversionError;
use lalrpop_util::lalrpop_mod;
use lexer::Lexer;
use std::io::{BufReader, Read};

lalrpop_mod!(
    #[allow(clippy::all)]
    parser,
    "/parser/parser.rs"
);

/// Parse an emblem source file at the given location.
pub fn parse_file<'ctx, 'input>(
    ctx: &'ctx mut Context,
    mut to_parse: SearchResult,
) -> Result<ParsedFile<'input>, Box<Error<'input>>>
where
    'ctx: 'input,
{
    let content = {
        let file = to_parse.file();
        let hint = file.len_hint();

        let mut reader = BufReader::new(file);
        let mut buf = hint
            .and_then(|len| usize::try_from(len).ok())
            .map(String::with_capacity)
            .unwrap_or_default();
        reader.read_to_string(&mut buf)?;
        buf
    };

    let file = {
        let path: String = to_parse
            .path
            .into_os_string()
            .into_string()
            .map_err(|s| OsStringConversionError::new(s))?;
        ctx.alloc_file(path, content)
    };

    parse(file.name(), file.content())
}

/// Parse a given string of emblem source code.
pub fn parse<'file>(
    name: &'file str,
    content: &'file str,
) -> Result<ParsedFile<'file>, Box<Error<'file>>> {
    let lexer = Lexer::new(name, content);
    let parser = parser::FileParser::new();

    Ok(parser.parse(lexer)?)
}

/// Create a string representation of a list of tokens which will fit in with surrounding text.
#[allow(dead_code)]
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
    use regex::Regex;

    use super::*;
    use crate::ast::AstDebug;

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

    fn assert_parse_error(name: &str, input: &str, expected: &str) {
        let re = Regex::new(&("^".to_owned() + expected)).unwrap();

        let inputs = [
            (name, input),
            (&format!("{} with newline", name), &format!("{}\n", input)),
        ];

        for (name, input) in inputs {
            let result = parse(name, input);
            assert!(result.is_err(), "{}: unexpected success", name);

            let err = result.unwrap_err();
            let err = err.parse_error();
            assert!(err.is_some(), "{}: expected error", name);

            let msg = err
                .unwrap()
                .to_string()
                .replace("Unrecognized", "Unrecognised");
            assert!(
                !expected.is_empty() && re.is_match(&msg),
                "{}: unexpected error:\n{}\n\nexpected message to start with:\n{}",
                name,
                msg,
                expected,
            );
        }
    }

    mod orphans {
        use super::*;

        #[test]
        fn general() {
            let tests = [
                ("open brace", "{"),
                ("close brace", "}"),
                ("colon", ":"),
                ("double-colon", "::"),
            ];

            for (name, tok) in tests {
                let expected = if tok == "{" || tok == "}" {
                    format!(
                        "Unrecognised token `\\{}` found at 1:1:1:{}",
                        tok,
                        1 + tok.len()
                    )
                } else {
                    format!(
                        "Unrecognised token `{}` found at 1:1:1:{}",
                        tok,
                        1 + tok.len()
                    )
                };

                assert_parse_error(name, tok, &expected);
            }
        }

        #[test]
        fn multi_line_comments() {
            assert_parse_error(
                "multi-line comment open",
                "/*",
                r"unclosed comment found at multi-line comment open[^\n]*:1:1-2",
            );
            assert_parse_error(
                "multi-line comment close",
                "*/",
                r"no comment to close found at[^\n]*1:1-2",
            );
        }
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
            assert_structure("command", ".order-66", "File[Par[[.order-66]]]")
        }

        #[test]
        fn inline_args() {
            assert_structure(
                "sole",
                ".exec{order66}",
                "File[Par[[.exec{[Word(order66)]}]]]",
            );
            assert_structure("start of line", ".old-man-say{leave her Johnny, leave her} tomorrow ye will get your pay", "File[Par[[.old-man-say{[Word(leave)|< >|Word(her)|< >|Word(Johnny,)|< >|Word(leave)|< >|Word(her)]}|< >|Word(tomorrow)|< >|Word(ye)|< >|Word(will)|< >|Word(get)|< >|Word(your)|< >|Word(pay)]]]");
            assert_structure("end of line", "I hate to .sail{on this rotten tub}", "File[Par[[Word(I)|< >|Word(hate)|< >|Word(to)|< >|.sail{[Word(on)|< >|Word(this)|< >|Word(rotten)|< >|Word(tub)]}]]]");
            assert_structure("middle of line", "For the .voyage-is{foul} and the winds don't blow", "File[Par[[Word(For)|< >|Word(the)|< >|.voyage-is{[Word(foul)]}|< >|Word(and)|< >|Word(the)|< >|Word(winds)|< >|Word(don't)|< >|Word(blow)]]]");
            assert_structure("nested", ".no{grog .allowed{and} rotten grub}", "File[Par[[.no{[Word(grog)|< >|.allowed{[Word(and)]}|< >|Word(rotten)|< >|Word(grub)]}]]]");

            assert_parse_error(
                "newline in brace-arg",
                ".order66{\n}",
                "newline in braced args found at newline in brace-arg[^:]*:1:9",
            );
            assert_parse_error(
                "newline in brace-arg",
                ".order66{general\nkenobi}",
                "newline in braced args found at newline in brace-arg[^:]*:1:9",
            );
            assert_parse_error(
                "par-break in brace-arg",
                ".order66{\n\n}",
                "newline in braced args found at par-break in brace-arg[^:]*:1:9",
            );
            assert_parse_error(
                "par-break in brace-arg",
                ".order66{general\n\nkenobi}",
                "newline in braced args found at par-break in brace-arg[^:]*:1:9",
            );
        }

        #[test]
        fn remainder_args() {
            assert_structure("start of line", ".now{we are ready}: to sail for the horn", "File[Par[[.now{[Word(we)|< >|Word(are)|< >|Word(ready)]}:[Word(to)|< >|Word(sail)|< >|Word(for)|< >|Word(the)|< >|Word(horn)]]]]");
            assert_structure(
                "middle of line",
                "our boots .and{our clothes boys}, are all in the pawn",
                "File[Par[[Word(our)|< >|Word(boots)|< >|.and{[Word(our)|< >|Word(clothes)|< >|Word(boys)]}|Word(,)|< >|Word(are)|< >|Word(all)|< >|Word(in)|< >|Word(the)|< >|Word(pawn)]]]",
            );
            assert_structure(
                "nested",
                "the anchor's on board .and{the cable's}: .all: stored",
                "File[Par[[Word(the)|< >|Word(anchor's)|< >|Word(on)|< >|Word(board)|< >|.and{[Word(the)|< >|Word(cable's)]}:[.all:[Word(stored)]]]]]",
            );
            assert_structure("nested in braces", "Heave away, bullies, .you{parish-rigged bums, .take: your hands from your pockets and don’t}: suck your thumbs", "File[Par[[Word(Heave)|< >|Word(away,)|< >|Word(bullies,)|< >|.you{[Word(parish)|-|Word(rigged)|< >|Word(bums,)|< >|.take:[Word(your)|< >|Word(hands)|< >|Word(from)|< >|Word(your)|< >|Word(pockets)|< >|Word(and)|< >|Word(don’t)]]}:[Word(suck)|< >|Word(your)|< >|Word(thumbs)]]]]");
            assert_structure("stacked", ".heave{a pawl}:, o heave away\n.way{hay}: roll 'an go!", "File[Par[[.heave{[Word(a)|< >|Word(pawl)]}:[Word(,)|< >|Word(o)|< >|Word(heave)|< >|Word(away)]]|[.way{[Word(hay)]}:[Word(roll)|< >|Word('an)|< >|Word(go!)]]]]");

            assert_parse_error(
                "sole at end of line",
                ".randy-dandy-o:",
                "Unrecognised EOF found at (1:16|2:1)",
            );
            assert_parse_error(
                "end of line",
                "randy .dandy-o:",
                "Unrecognised token `newline` found at 1:1[56]",
            );
        }

        #[test]
        fn trailer_args() {
            struct TrailerTest<'n, 'd, 'e> {
                name: &'n str,
                data: &'d [&'d str],
                expected_structure: &'e str,
            }

            let tests = [
                TrailerTest {
                    name: "one par per trailer arg",
                    data: &[
                        ".come{all you}:",
                        "\tyoung sailor men",
                        "\tlisten to me",
                        "::",
                        "\tI'll sing you a song",
                        "\tof the fish in the sea",
                    ],
                    expected_structure: "File[Par[.come{[Word(all)|< >|Word(you)]}::[Par[[Word(young)|< >|Word(sailor)|< >|Word(men)]|[Word(listen)|< >|Word(to)|< >|Word(me)]]]::[Par[[Word(I'll)|< >|Word(sing)|< >|Word(you)|< >|Word(a)|< >|Word(song)]|[Word(of)|< >|Word(the)|< >|Word(fish)|< >|Word(in)|< >|Word(the)|< >|Word(sea)]]]]]",
                },
                TrailerTest {
                    name: "two pars per trailer arg",
                    data: &[
                        ".come{all you}:",
                        "\tyoung sailor men",
                        "\t",
                        "\tlisten to me",
                        "::",
                        "\tI'll sing you a song",
                        "",
                        "\tof the fish in the sea",
                    ],
                    expected_structure: "File[Par[.come{[Word(all)|< >|Word(you)]}::[Par[[Word(young)|< >|Word(sailor)|< >|Word(men)]]|Par[[Word(listen)|< >|Word(to)|< >|Word(me)]]]::[Par[[Word(I'll)|< >|Word(sing)|< >|Word(you)|< >|Word(a)|< >|Word(song)]]|Par[[Word(of)|< >|Word(the)|< >|Word(fish)|< >|Word(in)|< >|Word(the)|< >|Word(sea)]]]]]",
                },
                TrailerTest {
                    name: "nested trailers",
                    data: &[
                        ".and{it's}:",
                        "\twindy weather, boys,",
                        "\t.stormy-weather{boys}:",
                        "\t\twhen the wind blows,",
                        "\t::",
                        "\t\twe're all together, boys",
                        "\t\tblow ye winds westerly",
                        "",
                        "\t.blow{ye}:",
                        "\t\twinds blow",
                        "",
                        "\t\tjolly sou'wester, boys",
                        "\t\t.steady{she goes}:",
                        "\t\t\tup jumps the eeo with his slippery tail",
                        "\t\tclimbs up aloft and reefs the topsail",
                        "",
                        "\tthen up jumps the shark .with: his nine rows of teeth,",
                        "\t.saying: you eat the dough boys,",
                        "\t.and{I'll eat}: the beef!",
                    ],
                    expected_structure: "File[Par[.and{[Word(it's)]}::[Par[[Word(windy)|< >|Word(weather,)|< >|Word(boys,)]|.stormy-weather{[Word(boys)]}::[Par[[Word(when)|< >|Word(the)|< >|Word(wind)|< >|Word(blows,)]]]::[Par[[Word(we're)|< >|Word(all)|< >|Word(together,)|< >|Word(boys)]|[Word(blow)|< >|Word(ye)|< >|Word(winds)|< >|Word(westerly)]]]]|Par[.blow{[Word(ye)]}::[Par[[Word(winds)|< >|Word(blow)]]|Par[[Word(jolly)|< >|Word(sou'wester,)|< >|Word(boys)]|.steady{[Word(she)|< >|Word(goes)]}::[Par[[Word(up)|< >|Word(jumps)|< >|Word(the)|< >|Word(eeo)|< >|Word(with)|< >|Word(his)|< >|Word(slippery)|< >|Word(tail)]]]|[Word(climbs)|< >|Word(up)|< >|Word(aloft)|< >|Word(and)|< >|Word(reefs)|< >|Word(the)|< >|Word(topsail)]]]]|Par[[Word(then)|< >|Word(up)|< >|Word(jumps)|< >|Word(the)|< >|Word(shark)|< >|.with:[Word(his)|< >|Word(nine)|< >|Word(rows)|< >|Word(of)|< >|Word(teeth,)]]|[.saying:[Word(you)|< >|Word(eat)|< >|Word(the)|< >|Word(dough)|< >|Word(boys,)]]|[.and{[Word(I'll)|< >|Word(eat)]}:[Word(the)|< >|Word(beef!)]]]]]]",
                },
                TrailerTest {
                    name: "remainder in trailer",
                    data: &[
                        ".up{jumps the .whale{the .largest{of}: all}}:",
                        "\tif you want any wind, I'll .blow{ye's}: a squall",
                    ],
                    expected_structure: "File[Par[.up{[Word(jumps)|< >|Word(the)|< >|.whale{[Word(the)|< >|.largest{[Word(of)]}:[Word(all)]]}]}::[Par[[Word(if)|< >|Word(you)|< >|Word(want)|< >|Word(any)|< >|Word(wind,)|< >|Word(I'll)|< >|.blow{[Word(ye's)]}:[Word(a)|< >|Word(squall)]]]]]]",
                },
                TrailerTest {
                    name: "stacked trailers",
                    data: &[
                        ".four:",
                        "\tand twenty British sailors",
                        ".met:",
                        "\thim on the king's highway",
                        "",
                        ".as:",
                        "\the went to be married",
                        ".pressed{he was}:",
                        "\tand sent away",
                    ],
                    expected_structure: "File[Par[.four::[Par[[Word(and)|< >|Word(twenty)|< >|Word(British)|< >|Word(sailors)]]]|.met::[Par[[Word(him)|< >|Word(on)|< >|Word(the)|< >|Word(king's)|< >|Word(highway)]]]]|Par[.as::[Par[[Word(he)|< >|Word(went)|< >|Word(to)|< >|Word(be)|< >|Word(married)]]]|.pressed{[Word(he)|< >|Word(was)]}::[Par[[Word(and)|< >|Word(sent)|< >|Word(away)]]]]]",
                },
            ];
            for test in &tests {
                let name_with_tabs = format!("{} (with tabs)", test.name);
                let data_with_tabs = test.data.join("\n");
                assert_structure(&name_with_tabs, &data_with_tabs, test.expected_structure);

                let name_with_spaces = format!("{} (with spaces)", test.name);
                let data_with_spaces = test
                    .data
                    .iter()
                    .map(|l| l.replace('\t', "    "))
                    .collect::<Vec<_>>()
                    .join("\n");
                assert_structure(
                    &name_with_spaces,
                    &data_with_spaces,
                    test.expected_structure,
                );
            }

            assert_parse_error(
                "end of populated line",
                &[
                    "william taylor was a .brisk{young sailor}:",
                    "\tfull of heart and full of play",
                ]
                .join("\n"),
                "Unrecognised token `newline` found at 1:43:2:1",
            );
            assert_parse_error(
                "missing indent",
                &[".until{did his mind uncover}:", "to a youthful lady gay"].join("\n"),
                "Unrecognised token `word` found at 2:1:2:3",
            );
            assert_parse_error(
                "missing second trailer",
                &[
                    ".until{did his mind uncover}:",
                    "\tto a youthful lady gay",
                    "::",
                    "\tfour and twenty british sailors",
                    "::",
                ]
                .join("\n"),
                "Unrecognised EOF found at (5:3|6:1)",
            );
        }

        #[test]
        fn attrs() {
            assert_structure(
                "empty",
                "we are .outward-bound[]",
                "File[Par[[Word(we)|< >|Word(are)|< >|.outward-bound[]]]]",
            );
            assert_structure(
                "unnamed-only",
                "we are .outward-bound[for,kingston,town]",
                "File[Par[[Word(we)|< >|Word(are)|< >|.outward-bound[(for)|(kingston)|(town)]]]]",
            );
            assert_structure("unnamed-only-with-spaces", "we are .outward-bound[ for , kingston , town ]", "File[Par[[Word(we)|< >|Word(are)|< >|.outward-bound[( for )|( kingston )|( town )]]]]");
            assert_structure(
                "unnamed-only-with-tabs",
                "we are .outward-bound[\tfor\t,\tkingston\t,\ttown\t]",
                r"File[Par[[Word(we)|< >|Word(are)|< >|.outward-bound[(\tfor\t)|(\tkingston\t)|(\ttown\t)]]]]",
            );

            assert_structure(
                "named",
                "we are .outward-bound[for=kingston,town]",
                "File[Par[[Word(we)|< >|Word(are)|< >|.outward-bound[(for)=(kingston)|(town)]]]]",
            );
            assert_structure("named-with-spaces", "we are .outward-bound[   for   =   kingston   ,   town   ]", "File[Par[[Word(we)|< >|Word(are)|< >|.outward-bound[(   for   )=(   kingston   )|(   town   )]]]]");
            assert_structure(
                "named-with-spaces",
                "we are .outward-bound[\tfor\t=\tkingston\t,\ttown\t]",
                r"File[Par[[Word(we)|< >|Word(are)|< >|.outward-bound[(\tfor\t)=(\tkingston\t)|(\ttown\t)]]]]",
            );

            assert_structure(
                "with-inline-args",
                "and we'll .heave[the,old=wheel,round]{and}{round}",
                r"File[Par[[Word(and)|< >|Word(we'll)|< >|.heave[(the)|(old)=(wheel)|(round)]{[Word(and)]}{[Word(round)]}]]]",
            );
            assert_structure(
                "with-trailer-args",
                "and we'll .heave[the,old=wheel,round]: and round",
                r"File[Par[[Word(and)|< >|Word(we'll)|< >|.heave[(the)|(old)=(wheel)|(round)]:[Word(and)|< >|Word(round)]]]]",
            );
            assert_structure(
                "with-remainder-args",
                ".heave[the,old=wheel,round]:\n\tand\n::\n\tround",
                r"File[Par[.heave[(the)|(old)=(wheel)|(round)]::[Par[[Word(and)]]]::[Par[[Word(round)]]]]]",
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
                "no gap",
                "to me!//to you!\n",
                r"File[Par[[Word(to)|< >|Word(me!)|//to you!]]]",
            );
            assert_structure(
                "space-after-comment",
                "to me!// to you!\n",
                r"File[Par[[Word(to)|< >|Word(me!)|// to you!]]]",
            );
            assert_structure(
                "space-after-comment",
                "to me!//\tto you!\n",
                r"File[Par[[Word(to)|< >|Word(me!)|//\tto you!]]]",
            );
            assert_structure(
                "space-before-comment",
                "to me! //to you!\n",
                "File[Par[[Word(to)|< >|Word(me!)|< >|//to you!]]]",
            );
            assert_structure(
                "tab-before-comment",
                "to me!\t//to you!\n",
                r"File[Par[[Word(to)|< >|Word(me!)|<\t>|//to you!]]]",
            );
            assert_structure(
                "surrounding-spaces",
                "to me! // to you!\n",
                r"File[Par[[Word(to)|< >|Word(me!)|< >|// to you!]]]",
            );
            assert_structure(
                "surrounding-tabs",
                "to me!\t//\tto you!\n",
                r"File[Par[[Word(to)|< >|Word(me!)|<\t>|//\tto you!]]]",
            );
            assert_structure(
                "surrounding-mix-1",
                "to me! //\tto you!\n",
                r"File[Par[[Word(to)|< >|Word(me!)|< >|//\tto you!]]]",
            );
            assert_structure(
                "surrounding-mix-2",
                "to me!\t// to you!\n",
                r"File[Par[[Word(to)|< >|Word(me!)|<\t>|// to you!]]]",
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
                "multiple-comments",
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
        fn as_trailer_arg() {
            assert_structure(
                "comment as sole arg",
                ".spaghetti:\n\t//and meatballs",
                "File[Par[.spaghetti::[Par[[//and meatballs]]]]]",
            );
        }
    }

    mod multi_line_comments {
        use super::*;

        #[test]
        fn empty() {
            assert_structure("single", "/**/", r"File[Par[[/*[]*/]]]");
            assert_structure("multiple", "/**//**/", r"File[Par[[/*[]*/|/*[]*/]]]");
            assert_structure(
                "multiple with space gap",
                "/**/ /**/",
                r"File[Par[[/*[]*/|< >|/*[]*/]]]",
            );
            assert_structure(
                "multiple with tab gap",
                "/**/\t/**/",
                r"File[Par[[/*[]*/|<\t>|/*[]*/]]]",
            );
            assert_structure("stacked", "/**/\n/**/", r"File[Par[[/*[]*/]|[/*[]*/]]]");
            assert_structure(
                "pars with stacked",
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
        fn unmatched() {
            assert_parse_error(
                "open",
                "/*spaghetti/*and*/meatballs",
                "unclosed comment found at open[^:]*:1:1-2",
            );
            assert_parse_error(
                "close",
                "spaghetti/*and*/meatballs */",
                "no comment to close found at close[^:]*:1:27-28",
            );
        }

        #[test]
        fn as_trailer_arg() {
            assert_structure(
                "comment as sole arg",
                ".spaghetti:\n\t/*and meatballs*/",
                "File[Par[.spaghetti::[Par[[/*[and meatballs]*/]]]]]",
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
        }

        #[test]
        fn before_remainder_args() {
            assert_structure(
                "single-line",
                "/*spaghetti*/.and: meatballs",
                "File[Par[[/*[spaghetti]*/|.and:[Word(meatballs)]]]]",
            );
            assert_structure(
                "multi-line",
                "/*spaghetti\n\t\t*/.and: meatballs",
                r"File[Par[[/*[spaghetti|\n|\t\t]*/|.and:[Word(meatballs)]]]]",
            );
        }

        #[test]
        fn before_trailer_args() {
            assert_parse_error(
                "trailer-args",
                "/*spaghetti*/.and:\n\tmeatballs",
                "Unrecognised token `newline` found at 1:19",
            );
        }
    }
}
