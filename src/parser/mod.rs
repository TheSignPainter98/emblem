pub mod error;
pub mod lexer;
pub mod location;
mod location_context;
mod point;

pub use error::Error;
pub use lexer::LexicalError;
pub use location::Location;
pub use location_context::LocationContext;
pub use point::Point;

use crate::args::SearchResult;
use crate::ast;
use crate::context::Context;
use ast::parsed::ParsedFile;
use error::StringConversionError;
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
        let mut path: String = to_parse
            .path
            .into_os_string()
            .into_string()
            .map_err(StringConversionError::new)?;
        if path == "-" {
            path = "(stdin)".into();
        }
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

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::ast::AstDebug;
    use regex::Regex;

    pub fn assert_structure(name: &str, input: &str, expected: &str) {
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
                r#"unclosed comment found at \["multi-line comment open[^\n]*:1:1-2"\]"#,
            );
            assert_parse_error(
                "multi-line comment open",
                "/*/*",
                r#"unclosed comment found at \["multi-line comment open[^\n]*:1:1-2", "multi-line comment open[^\n]*:1:3-4"\]"#,
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

        fn ast_debug_pluses(n: usize) -> String {
            let s = "+".repeat(n);
            if s.is_empty() {
                s
            } else {
                format!("({s})")
            }
        }

        #[test]
        fn command_only() {
            for num_pluses in 0..=3 {
                assert_structure(
                    "command",
                    &format!(".order-66{}", "+".repeat(num_pluses)),
                    &format!("File[Par[[.order-66{}]]]", ast_debug_pluses(num_pluses)),
                );
            }
        }

        #[test]
        fn inline_args() {
            for num_pluses in 0..=3 {
                let pluses = "+".repeat(num_pluses);
                let ast_pluses = ast_debug_pluses(num_pluses);
                assert_structure(
                    "sole",
                    &format!(".exec{}{{order66}}", pluses),
                    &format!("File[Par[[.exec{}{{[Word(order66)]}}]]]", ast_pluses),
                );
                assert_structure(
                    "start of line",
                    &format!(".old-man-say{}{{leave her Johnny, leave her}} tomorrow ye will get your pay", pluses),
                    &format!("File[Par[[.old-man-say{}{{[Word(leave)|< >|Word(her)|< >|Word(Johnny,)|< >|Word(leave)|< >|Word(her)]}}|< >|Word(tomorrow)|< >|Word(ye)|< >|Word(will)|< >|Word(get)|< >|Word(your)|< >|Word(pay)]]]", ast_pluses)
                );
                assert_structure(
                    "end of line",
                    &format!("I hate to .sail{}{{on this rotten tub}}", pluses),
                    &format!("File[Par[[Word(I)|< >|Word(hate)|< >|Word(to)|< >|.sail{}{{[Word(on)|< >|Word(this)|< >|Word(rotten)|< >|Word(tub)]}}]]]", ast_pluses),
                );
                assert_structure(
                    "middle of line",
                    &format!("For the .voyage-is{}{{foul}} and the winds don't blow", pluses),
                    &format!("File[Par[[Word(For)|< >|Word(the)|< >|.voyage-is{}{{[Word(foul)]}}|< >|Word(and)|< >|Word(the)|< >|Word(winds)|< >|Word(don't)|< >|Word(blow)]]]", ast_pluses),
                );
                assert_structure(
                    "nested",
                    &format!(".no{}{{grog .allowed{}{{and}} rotten grub}}", pluses, pluses),
                    &format!("File[Par[[.no{}{{[Word(grog)|< >|.allowed{}{{[Word(and)]}}|< >|Word(rotten)|< >|Word(grub)]}}]]]", ast_pluses, ast_pluses),
                );

                assert_parse_error(
                    "newline in brace-arg",
                    &format!(".order66{}{{\n}}", pluses),
                    &format!(
                        "newline in braced args found at newline in brace-arg[^:]*:1:{}",
                        9 + num_pluses
                    ),
                );
                assert_parse_error(
                    "newline in brace-arg",
                    &format!(".order66{}{{general\nkenobi}}", pluses),
                    &format!(
                        "newline in braced args found at newline in brace-arg[^:]*:1:{}",
                        9 + num_pluses
                    ),
                );
                assert_parse_error(
                    "par-break in brace-arg",
                    &format!(".order66{}{{\n\n}}", pluses),
                    &format!(
                        "newline in braced args found at par-break in brace-arg[^:]*:1:{}",
                        9 + num_pluses
                    ),
                );
                assert_parse_error(
                    "par-break in brace-arg",
                    &format!(".order66{}{{general\n\nkenobi}}", pluses),
                    &format!(
                        "newline in braced args found at par-break in brace-arg[^:]*:1:{}",
                        9 + num_pluses
                    ),
                );
            }
        }

        #[test]
        fn remainder_args() {
            for num_pluses in 0..=3 {
                let pluses = "+".repeat(num_pluses);
                let ast_pluses = ast_debug_pluses(num_pluses);
                assert_structure(
                    "start of line",
                    &format!(".now{}{{we are ready}}: to sail for the horn", pluses),
                    &format!("File[Par[[.now{}{{[Word(we)|< >|Word(are)|< >|Word(ready)]}}:[Word(to)|< >|Word(sail)|< >|Word(for)|< >|Word(the)|< >|Word(horn)]]]]", ast_pluses),
                );
                assert_structure(
                    "middle of line",
                    &format!("our boots .and{}{{our clothes boys}}, are all in the pawn", pluses),
                    &format!("File[Par[[Word(our)|< >|Word(boots)|< >|.and{}{{[Word(our)|< >|Word(clothes)|< >|Word(boys)]}}|Word(,)|< >|Word(are)|< >|Word(all)|< >|Word(in)|< >|Word(the)|< >|Word(pawn)]]]", ast_pluses),
                );
                assert_structure(
                    "nested",
                    &format!("the anchor's on board .and{}{{the cable's}}: .all: stored", pluses),
                    &format!("File[Par[[Word(the)|< >|Word(anchor's)|< >|Word(on)|< >|Word(board)|< >|.and{}{{[Word(the)|< >|Word(cable's)]}}:[.all:[Word(stored)]]]]]", ast_pluses),
                );
                assert_structure(
                    "nested in braces",
                    &format!("Heave away, bullies, .you{}{{parish-rigged bums, .take: your hands from your pockets and don’t}}: suck your thumbs", pluses),
                    &format!("File[Par[[Word(Heave)|< >|Word(away,)|< >|Word(bullies,)|< >|.you{}{{[Word(parish)|-|Word(rigged)|< >|Word(bums,)|< >|.take:[Word(your)|< >|Word(hands)|< >|Word(from)|< >|Word(your)|< >|Word(pockets)|< >|Word(and)|< >|Word(don’t)]]}}:[Word(suck)|< >|Word(your)|< >|Word(thumbs)]]]]", ast_pluses),
                );
                assert_structure(
                    "stacked",
                    &format!(".heave{}{{a pawl}}:, o heave away\n.way{}{{hay}}: roll 'an go!", pluses, pluses),
                    &format!("File[Par[[.heave{}{{[Word(a)|< >|Word(pawl)]}}:[Word(,)|< >|Word(o)|< >|Word(heave)|< >|Word(away)]]|[.way{}{{[Word(hay)]}}:[Word(roll)|< >|Word('an)|< >|Word(go!)]]]]", ast_pluses, ast_pluses),
                );

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
        }

        #[test]
        fn trailer_args() {
            struct TrailerTest<'n, 'd, 'e> {
                name: &'n str,
                data: &'d [&'d str],
                expected_structure: &'e str,
            }

            for num_pluses in 0..=3 {
                let pluses = "+".repeat(num_pluses);
                let ast_pluses = ast_debug_pluses(num_pluses);
                let tests = [
                    TrailerTest {
                        name: "one par per trailer arg",
                        data: &[
                            &format!(".come{}{{all you}}:", pluses),
                            "\tyoung sailor men",
                            "\tlisten to me",
                            "::",
                            "\tI'll sing you a song",
                            "\tof the fish in the sea",
                        ],
                        expected_structure: &format!("File[Par[.come{}{{[Word(all)|< >|Word(you)]}}::[Par[[Word(young)|< >|Word(sailor)|< >|Word(men)]|[Word(listen)|< >|Word(to)|< >|Word(me)]]]::[Par[[Word(I'll)|< >|Word(sing)|< >|Word(you)|< >|Word(a)|< >|Word(song)]|[Word(of)|< >|Word(the)|< >|Word(fish)|< >|Word(in)|< >|Word(the)|< >|Word(sea)]]]]]", ast_pluses),
                    },
                    TrailerTest {
                        name: "two pars per trailer arg",
                        data: &[
                            &format!(".come{}{{all you}}:", pluses),
                            "\tyoung sailor men",
                            "\t",
                            "\tlisten to me",
                            "::",
                            "\tI'll sing you a song",
                            "",
                            "\tof the fish in the sea",
                        ],
                        expected_structure: &format!("File[Par[.come{}{{[Word(all)|< >|Word(you)]}}::[Par[[Word(young)|< >|Word(sailor)|< >|Word(men)]]|Par[[Word(listen)|< >|Word(to)|< >|Word(me)]]]::[Par[[Word(I'll)|< >|Word(sing)|< >|Word(you)|< >|Word(a)|< >|Word(song)]]|Par[[Word(of)|< >|Word(the)|< >|Word(fish)|< >|Word(in)|< >|Word(the)|< >|Word(sea)]]]]]", ast_pluses),
                    },
                    TrailerTest {
                        name: "nested trailers",
                        data: &[
                            &format!(".and{}{{it's}}:", pluses),
                            "\twindy weather, boys,",
                            &format!("\t.stormy-weather{}{{boys}}:", pluses),
                            "\t\twhen the wind blows,",
                            "\t::",
                            "\t\twe're all together, boys",
                            "\t\tblow ye winds westerly",
                            "",
                            &format!("\t.blow{}{{ye}}:", pluses),
                            "\t\twinds blow",
                            "",
                            "\t\tjolly sou'wester, boys",
                            &format!("\t\t.steady{}{{she goes}}:", pluses),
                            "\t\t\tup jumps the eeo with his slippery tail",
                            "\t\tclimbs up aloft and reefs the topsail",
                            "",
                            "\tthen up jumps the shark .with: his nine rows of teeth,",
                            "\t.saying: you eat the dough boys,",
                            &format!("\t.and{}{{I'll eat}}: the beef!", pluses),
                        ],
                        expected_structure: &format!("File[Par[.and{}{{[Word(it's)]}}::[Par[[Word(windy)|< >|Word(weather,)|< >|Word(boys,)]|.stormy-weather{}{{[Word(boys)]}}::[Par[[Word(when)|< >|Word(the)|< >|Word(wind)|< >|Word(blows,)]]]::[Par[[Word(we're)|< >|Word(all)|< >|Word(together,)|< >|Word(boys)]|[Word(blow)|< >|Word(ye)|< >|Word(winds)|< >|Word(westerly)]]]]|Par[.blow{}{{[Word(ye)]}}::[Par[[Word(winds)|< >|Word(blow)]]|Par[[Word(jolly)|< >|Word(sou'wester,)|< >|Word(boys)]|.steady{}{{[Word(she)|< >|Word(goes)]}}::[Par[[Word(up)|< >|Word(jumps)|< >|Word(the)|< >|Word(eeo)|< >|Word(with)|< >|Word(his)|< >|Word(slippery)|< >|Word(tail)]]]|[Word(climbs)|< >|Word(up)|< >|Word(aloft)|< >|Word(and)|< >|Word(reefs)|< >|Word(the)|< >|Word(topsail)]]]]|Par[[Word(then)|< >|Word(up)|< >|Word(jumps)|< >|Word(the)|< >|Word(shark)|< >|.with:[Word(his)|< >|Word(nine)|< >|Word(rows)|< >|Word(of)|< >|Word(teeth,)]]|[.saying:[Word(you)|< >|Word(eat)|< >|Word(the)|< >|Word(dough)|< >|Word(boys,)]]|[.and{}{{[Word(I'll)|< >|Word(eat)]}}:[Word(the)|< >|Word(beef!)]]]]]]", ast_pluses, ast_pluses, ast_pluses, ast_pluses, ast_pluses),
                    },
                    TrailerTest {
                        name: "remainder in trailer",
                        data: &[
                            &format!(".up{}{{jumps the .whale{}{{the .largest{}{{of}}: all}}}}:", pluses, pluses, pluses),
                            &format!("\tif you want any wind, I'll .blow{}{{ye's}}: a squall", pluses),
                        ],
                        expected_structure: &format!("File[Par[.up{}{{[Word(jumps)|< >|Word(the)|< >|.whale{}{{[Word(the)|< >|.largest{}{{[Word(of)]}}:[Word(all)]]}}]}}::[Par[[Word(if)|< >|Word(you)|< >|Word(want)|< >|Word(any)|< >|Word(wind,)|< >|Word(I'll)|< >|.blow{}{{[Word(ye's)]}}:[Word(a)|< >|Word(squall)]]]]]]", ast_pluses, ast_pluses, ast_pluses, ast_pluses)
                    },
                    TrailerTest {
                        name: "stacked trailers",
                        data: &[
                            &format!(".four{}:", pluses),
                            "\tand twenty British sailors",
                            &format!(".met{}:", pluses),
                            "\thim on the king's highway",
                            "",
                            &format!(".as{}:", pluses),
                            "\the went to be married",
                            &format!(".pressed{}{{he was}}:", pluses),
                            "\tand sent away",
                        ],
                        expected_structure: &format!("File[Par[.four{}::[Par[[Word(and)|< >|Word(twenty)|< >|Word(British)|< >|Word(sailors)]]]|.met{}::[Par[[Word(him)|< >|Word(on)|< >|Word(the)|< >|Word(king's)|< >|Word(highway)]]]]|Par[.as{}::[Par[[Word(he)|< >|Word(went)|< >|Word(to)|< >|Word(be)|< >|Word(married)]]]|.pressed{}{{[Word(he)|< >|Word(was)]}}::[Par[[Word(and)|< >|Word(sent)|< >|Word(away)]]]]]", ast_pluses, ast_pluses, ast_pluses, ast_pluses),
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

            assert_parse_error("unclosed", ".heave[", "(unexpected EOF found at 1:8|newline in attributes found at unclosed with newline:1:7-7)");
            assert_parse_error(
                "unexpected open bracket",
                ".heave[[",
                r"(unexpected EOF found at 1:9|unexpected character '\[' found at unexpected open bracket[^:]*:1:8-8)",
            );
        }
    }

    mod interword {
        use super::*;

        struct InterwordTest {
            input: String,
            expected: String,
        }

        impl InterwordTest {
            fn run(&self, name: &str) {
                assert_structure(name, &self.input, &self.expected);
            }
        }

        fn test_dash(name: &str, dash: &str, repr: &str) {
            let tests = vec![
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
            for test in tests {
                test.run(name);
            }
        }

        fn test_glue(name: &str, glue: &str, repr: &str) {
            let tests = vec![
                InterwordTest {
                    input: glue.into(),
                    expected: format!("File[Par[[SpiltGlue({})]]]", repr),
                },
                InterwordTest {
                    input: format!("a{}b", glue),
                    expected: format!("File[Par[[Word(a)|{}|Word(b)]]]", repr),
                },
                InterwordTest {
                    input: format!("a {}b", glue),
                    expected: format!("File[Par[[Word(a)|SpiltGlue( {})|Word(b)]]]", repr),
                },
                InterwordTest {
                    input: format!("a{} b", glue),
                    expected: format!("File[Par[[Word(a)|SpiltGlue({} )|Word(b)]]]", repr),
                },
                InterwordTest {
                    input: format!("a {} b", glue),
                    expected: format!("File[Par[[Word(a)|SpiltGlue( {} )|Word(b)]]]", repr),
                },
                InterwordTest {
                    input: format!("a\n{}b", glue),
                    expected: format!("File[Par[[Word(a)]|[SpiltGlue({})|Word(b)]]]", repr),
                },
                InterwordTest {
                    input: format!("a{}\nb", glue),
                    expected: format!("File[Par[[Word(a)|SpiltGlue({})]|[Word(b)]]]", repr),
                },
                InterwordTest {
                    input: format!("a{}", glue),
                    expected: format!("File[Par[[Word(a)|SpiltGlue({})]]]", repr),
                },
                InterwordTest {
                    input: format!("{}b", glue),
                    expected: format!("File[Par[[SpiltGlue({})|Word(b)]]]", repr),
                },
            ];
            for test in tests {
                test.run(name);
            }
        }

        #[test]
        fn hyphen() {
            test_dash("hyphen", "-", "-");
        }

        #[test]
        fn en() {
            test_dash("en", "--", "--");
        }

        #[test]
        fn em() {
            test_dash("em", "---", "---");
        }

        #[test]
        fn glue() {
            test_glue("em", "~", "~");
        }

        #[test]
        fn nbsp() {
            test_glue("em", "~~", "~~");
        }

        #[test]
        fn mixed() {
            test_dash("em-hyph", "----", "---|-");
            test_dash("em-en", "-----", "---|--");
            test_dash("em-em", "------", "---|---");

            fn test_mix(name: &str, to_test: &str, repr: &str) {
                assert_structure(
                    name,
                    &format!("a{to_test}b"),
                    &format!("File[Par[[Word(a)|{repr}|Word(b)]]]"),
                );
            }

            let glues = [
                ("~", "~"),
                ("~~", "~~"),
                ("~ ", "SpiltGlue(~ )"),
                (" ~", "SpiltGlue( ~)"),
                (" ~ ", "SpiltGlue( ~ )"),
            ];
            for (raw, repr) in glues {
                test_mix(
                    "glue-mixed-1-dash-1",
                    &format!("{raw}-",),
                    &format!("{repr}|-"),
                );
                test_mix(
                    "glue-mixed-1-dash-2",
                    &format!("-{raw}",),
                    &format!("-|{repr}"),
                );
                test_mix(
                    "glue-mixed-2-dashes-1",
                    &format!("{raw}--",),
                    &format!("{repr}|--"),
                );
                test_mix(
                    "glue-mixed-2-dashes-2",
                    &format!("-{raw}-",),
                    &format!("-|{repr}|-"),
                );
                test_mix(
                    "glue-mixed-2-dashes-3",
                    &format!("--{raw}",),
                    &format!("--|{repr}"),
                );
                test_mix(
                    "glue-mixed-3-dashes-1",
                    &format!("{raw}---",),
                    &format!("{repr}|---"),
                );
                test_mix(
                    "glue-mixed-3-dashes-2",
                    &format!("-{raw}--",),
                    &format!("-|{repr}|--"),
                );
                test_mix(
                    "glue-mixed-3-dashes-3",
                    &format!("--{raw}-",),
                    &format!("--|{repr}|-"),
                );
                test_mix(
                    "glue-mixed-3-dashes-4",
                    &format!("---{raw}",),
                    &format!("---|{repr}"),
                );
            }
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
                r#"unclosed comment found at \["open[^:]*:1:1-2"\]"#,
            );
            assert_parse_error(
                "open",
                "/*spaghetti/*and meatballs",
                r#"unclosed comment found at \["open[^:]*:1:1-2", "open[^:]*:1:12-13"\]"#,
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

    mod syntactic_sugar {
        use super::*;

        mod emph_delimiters {
            use super::*;

            static DELIMS: [(&str, &str); 7] = [
                ("_", "$it(_)"),
                ("*", "$it(*)"),
                ("__", "$bf(__)"),
                ("**", "$bf(**)"),
                ("`", "$tt"),
                ("=", "$sc"),
                ("==", "$af"),
            ];

            #[test]
            fn mixed_nesting() {
                for (outer, outer_repr) in &DELIMS {
                    let sanitised_outer = outer.replace('*', r"\*");
                    let outer_chars = {
                        let mut outer_chars: Vec<_> = outer.chars().collect();
                        outer_chars.sort();
                        outer_chars.dedup();
                        outer_chars
                    };

                    for (inner, inner_repr) in &DELIMS {
                        let sanitised_inner = inner.replace('*', r"\*");

                        assert_structure(
                            &format!("normal nesting {outer} and {inner}"),
                            &format!("{outer}spaghetti {inner}and{inner} meatballs{outer}"),
                            &format!("File[Par[[{outer_repr}{{[Word(spaghetti)|< >|{inner_repr}{{[Word(and)]}}|< >|Word(meatballs)]}}]]]"),
                        );

                        let inner_chars = {
                            let mut inner_chars: Vec<_> = inner.chars().collect();
                            inner_chars.sort();
                            inner_chars.dedup();
                            inner_chars
                        };

                        if outer_chars == inner_chars
                            && outer.len() >= inner.len()
                            && inner.len() != 2
                            && outer_chars != ['`']
                        {
                            // Check for troublesome nesting
                            assert_parse_error(
                                &format!("clash-nesting {inner} and {outer}"),
                                &format!("{outer}spaghetti {inner}and meatballs{inner}{outer}"),
                                &format!(
                                    r"delimiter mismatch for {sanitised_inner} found at clash-nesting {sanitised_inner} and {sanitised_outer}[^:]*:1:{}-{} \(failed to match at clash-nesting {sanitised_inner} and {sanitised_outer}[^:]*:1:{}-{}\)",
                                    24 + outer.len() + inner.len(),
                                    24 + outer.len() + 2 * inner.len(),
                                    11 + outer.len(),
                                    10 + outer.len() + inner.len()
                                ),
                            );
                        } else {
                            // Check nesting is okay
                            assert_structure(
                                &format!("chash-nesting nesting {outer} and meatballs{inner}"),
                                &format!("{outer}spaghetti {inner}and meatballs{inner}{outer}"),
                                &format!("File[Par[[{outer_repr}{{[Word(spaghetti)|< >|{inner_repr}{{[Word(and)|< >|Word(meatballs)]}}]}}]]]"),
                            );
                        }
                    }
                }
            }

            #[test]
            fn multi_line() {
                for (delim, _) in &DELIMS {
                    let sanitised = delim.replace('*', r"\*");
                    assert_parse_error(
                        &format!("multi-line {delim}"),
                        &format!("{delim}foo\nbar{delim}"),
                        &format!(
                            r#"newline in "{sanitised}" emphasis found at multi-line {sanitised}[^:]*:1:{}-2:1"#,
                            4 + delim.len()
                        ),
                    );
                }
            }

            #[test]
            fn mismatched() {
                for (left, _) in &DELIMS {
                    for (right, _) in &DELIMS {
                        if left == right {
                            continue;
                        }

                        let sanitised_left = left.replace('*', r"\*");
                        let sanitised_right = right.replace('*', r"\*");

                        assert_parse_error(
                            &format!("mismatch({left},{right})"),
                            &format!("{left}foo{right}"),
                            &format!(
                                r"delimiter mismatch for {sanitised_left} found at mismatch\({sanitised_left},{sanitised_right}\)[^:]*:1:{}-{} \(failed to match at mismatch\({sanitised_left},{sanitised_right}\)[^:]*:1:1-{}\)",
                                4 + left.len(),
                                3 + left.len() + right.len(),
                                left.len(),
                            ),
                        );
                    }
                }
            }
        }

        mod headings {
            use super::*;

            #[test]
            fn start_of_line() {
                for level in 1..=6 {
                    for pluses in 0..=2 {
                        assert_structure(
                            &format!("level:{level}, pluses:{pluses}"),
                            &format!("{}{} foo", "#".repeat(level), "+".repeat(pluses)),
                            &format!(
                                "File[Par[[$h{level}{}{{[Word(foo)]}}]]]",
                                if pluses > 0 {
                                    format!("({})", "+".repeat(pluses))
                                } else {
                                    "".into()
                                }
                            ),
                        );
                    }
                }

                assert_structure(
                    "nested inline args",
                    "## .bar{baz}",
                    "File[Par[[$h2{[.bar{[Word(baz)]}]}]]]",
                );
                assert_structure(
                    "nested remainder args",
                    "## .bar: baz",
                    "File[Par[[$h2{[.bar:[Word(baz)]]}]]]",
                );
                assert_parse_error(
                    "nested trailer args",
                    "## .foo:\n\tbar",
                    "Unrecognised token `newline` found at 1:9:2:1",
                );

                assert_parse_error(
                    "nested headings",
                    "## ## foo",
                    "unexpected heading at nested headings[^:]*:1:4-5",
                );
                assert_parse_error(
                    "no argument",
                    "##",
                    "Unrecognised token `newline` found at (1:1:1:3|1:3:2:1)",
                );
            }

            #[test]
            fn midline() {
                assert_parse_error(
                    "inline",
                    "foo ###+ bar",
                    "unexpected heading at inline[^:]*:1:5-8",
                );
                assert_parse_error(
                    "inline",
                    "foo .bar: ###+ baz",
                    "unexpected heading at inline[^:]*:1:11-14",
                );
            }
        }
    }
}
