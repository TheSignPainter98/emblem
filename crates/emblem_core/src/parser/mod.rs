pub mod error;
pub mod lexer;
pub mod location;
mod point;

pub use lexer::LexicalError;
pub use location::Location;
pub use point::Point;

use crate::context::Context;
use crate::path::SearchResult;
use crate::{ast, Error, FileContent, FileName, Result};
use ast::parsed::ParsedFile;
use lalrpop_util::lalrpop_mod;
use lexer::Lexer;
use std::io::{BufReader, Read};

lalrpop_mod!(
    #[allow(clippy::all)]
    parser,
    "/parser/parser.rs"
);

/// Parse an emblem source file at the given location.
pub fn parse_file(ctx: &Context, mut to_parse: SearchResult) -> Result<ParsedFile> {
    let file = {
        let raw = to_parse.path().as_os_str();
        let mut path: &str = to_parse
            .path()
            .as_os_str()
            .to_str()
            .ok_or_else(|| Error::string_conversion(raw.to_owned()))?;
        if path == "-" {
            path = "(stdin)";
        }
        ctx.alloc_file_name(path)
    };

    let content = {
        let file = to_parse.file();
        let hint = file.len_hint();

        let mut reader = BufReader::new(file);
        let mut buf = hint
            .and_then(|len| usize::try_from(len).ok())
            .map(String::with_capacity)
            .unwrap_or_default();
        reader.read_to_string(&mut buf)?;
        ctx.alloc_file_content(buf)
    };

    parse(file, content)
}

/// Parse a given string of emblem source code.
pub fn parse(file_name: FileName, content: FileContent) -> Result<ParsedFile> {
    let lexer = Lexer::new(file_name.clone(), content);
    let parser = parser::FileParser::new();

    parser
        .parse(lexer)
        .map_err(|cause| Error::parse(file_name, cause))
}

#[cfg(test)]
pub mod test {
    use std::borrow::Cow;

    use super::*;
    use crate::{ast::AstDebug, Doc};
    use regex::Regex;

    pub struct ParserTest {
        name: Cow<'static, str>,
        input: Option<Cow<'static, str>>,
        test_attempted: bool,
    }

    impl ParserTest {
        pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
            let name = name.into();
            Self {
                name,
                input: None,
                test_attempted: false,
            }
        }

        pub fn input(mut self, input: impl Into<Cow<'static, str>>) -> Self {
            self.input = Some({
                let mut input = input.into();
                if input.ends_with('\n') {
                    input = input[..input.len() - 1].to_string().into();
                }
                assert!(!input.ends_with('\n')); // Trailing newlines are added
                                                 // elsewhere
                input
            });
            self
        }

        fn produces_ast(mut self, structure_repr: impl AsRef<str>) {
            self.setup();

            let input = self.input.as_ref().unwrap();
            let expected_repr = StructureRepr::Ast(structure_repr.as_ref());
            self.assert_input_produces(&self.name, input, &expected_repr);
            self.assert_input_produces(
                &format!("{} with newline", self.name),
                &format!("{}\n", input),
                &expected_repr,
            );
        }

        pub fn produces_doc(mut self, structure_repr: impl AsRef<str>) {
            self.setup();

            let input = self.input.as_ref().unwrap();
            let expected = StructureRepr::Doc(structure_repr.as_ref());
            self.assert_input_produces(&self.name, input, &expected)
        }

        fn assert_valid(&self) {
            assert!(self.input.is_some(), "{}: test has no input!", self.name);
        }

        fn assert_input_produces(&self, name: &str, input: &str, expected: &StructureRepr<'_>) {
            let ctx = Context::test_new();
            let parse_result = self.parse(&ctx, name, input);
            let Ok(ast) = parse_result else {
                panic!(
                    "{}: expected Ok parse result when parsing {input:?}, got: {:?}",
                    self.name,
                    parse_result.unwrap_err()
                );
            };
            let (parsed_repr, expected_repr) = match expected {
                StructureRepr::Ast(expected_repr) => (ast.repr(), *expected_repr),
                StructureRepr::Doc(expected_repr) => (Doc::from(ast).repr(), *expected_repr),
            };
            assert_eq!(
                parsed_repr, expected_repr,
                "{}: unexpected structure",
                self.name
            )
        }

        fn causes(mut self, matches: impl AsRef<str>) {
            self.setup();

            let input = self.input.as_ref().unwrap();
            let matches = Regex::new(&("^".to_string() + matches.as_ref())).unwrap();
            self.assert_input_errors(&self.name, input, &matches);
            self.assert_input_errors(
                &format!("{} with newline", self.name),
                &format!("{}\n", input),
                &matches,
            );
        }

        fn assert_input_errors(&self, name: &str, input: &str, matches: &Regex) {
            let ctx = Context::test_new();
            let parse_result = self.parse(&ctx, name, input);
            assert!(parse_result.is_err(), "{}: unexpected success", name);
            let msg = parse_result.unwrap_err().to_string();
            assert!(
                msg.contains(name),
                "expected file name {name:?} in error messages {msg:?}"
            );

            let sanitised_msg = {
                let sanitised_msg = msg.replace("Unrecognized", "Unrecognised");
                // Remove file name
                let colon_idx = sanitised_msg.find(':').expect("no colon after file name");
                sanitised_msg[2 + colon_idx..].to_string()
            };
            assert!(
                matches.is_match(&sanitised_msg),
                "{}: unexpected error:\n{}\n\nexpected message to start with:\n{}",
                name,
                sanitised_msg,
                matches.as_str(),
            );
        }

        fn parse(&self, ctx: &Context, name: &str, input: &str) -> Result<ParsedFile> {
            let name = ctx.alloc_file_name(name);
            let input = ctx.alloc_file_content(input);
            super::parse(name, input)
        }

        fn setup(&mut self) {
            println!("testing {}...", self.name);

            self.test_attempted = true;

            self.assert_valid();
        }
    }

    impl Drop for ParserTest {
        fn drop(&mut self) {
            assert!(self.test_attempted, "test {} never examined!", self.name);
        }
    }

    #[derive(Debug)]
    enum StructureRepr<'r> {
        Ast(&'r str),
        Doc(&'r str),
    }

    mod shebang {
        use super::*;

        #[test]
        fn general() {
            ParserTest::new("empty")
                .input("#!")
                .produces_ast("File[Par[[Shebang()]]]");
            ParserTest::new("sole")
                .input("#!em build")
                .produces_ast("File[Par[[Shebang(em build)]]]");
        }

        #[test]
        fn whitespace_preserved() {
            ParserTest::new("space")
                .input("#! em build")
                .produces_ast(r"File[Par[[Shebang( em build)]]]");
            ParserTest::new("tab")
                .input("#!\tem build")
                .produces_ast(r"File[Par[[Shebang(\tem build)]]]");
        }

        #[test]
        fn only_at_start() {
            ParserTest::new("at-end")
                .input("#!foo\nbar\n#!baz")
                .causes("Unrecognised token `word` found at 3:2:3:6")
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

                ParserTest::new(name).input(tok).causes(expected);
            }
        }

        #[test]
        fn multi_line_comments() {
            ParserTest::new("multi-line comment open")
                .input("/*")
                .causes(r#"unclosed comment found at \["multi-line comment open[^\n]*:1:1-2"\]"#);
            ParserTest::new( "multi-line comment open")
                .input("/*/*")
                .causes(r#"unclosed comment found at \["multi-line comment open[^\n]*:1:1-2", "multi-line comment open[^\n]*:1:3-4"\]"#);
            ParserTest::new("multi-line comment close")
                .input("*/")
                .causes(r"no comment to close found at[^\n]*1:1-2");
        }
    }

    mod paragraphs {
        use super::*;

        #[test]
        fn empty() {
            ParserTest::new("empty")
                .input("")
                .produces_ast("File[Par[[]]]");
        }

        #[test]
        fn single_line() {
            ParserTest::new("single line")
                .input("hello, world!")
                .produces_ast("File[Par[[Word(hello,)|< >|Word(world!)]]]");
            ParserTest::new("single line with tabs")
                .input("hello,\tworld!")
                .produces_ast(r"File[Par[[Word(hello,)|<\t>|Word(world!)]]]");
            ParserTest::new("single line for many pars")
                .input(indoc::indoc!("
                    Spiderpig, Spiderpig,

                    Does whatever a Spiderpig does.

                    Can he swing from a web?

                    No, he can't, he's a pig,

                    Look out, he is a Spiderpig!
                "))
                .produces_ast("File[Par[[Word(Spiderpig,)|< >|Word(Spiderpig,)]]|Par[[Word(Does)|< >|Word(whatever)|< >|Word(a)|< >|Word(Spiderpig)|< >|Word(does.)]]|Par[[Word(Can)|< >|Word(he)|< >|Word(swing)|< >|Word(from)|< >|Word(a)|< >|Word(web?)]]|Par[[Word(No,)|< >|Word(he)|< >|Word(can't,)|< >|Word(he's)|< >|Word(a)|< >|Word(pig,)]]|Par[[Word(Look)|< >|Word(out,)|< >|Word(he)|< >|Word(is)|< >|Word(a)|< >|Word(Spiderpig!)]]]");
        }

        #[test]
        fn multiple_lines() {
            ParserTest::new("multiple lines")
                .input(indoc::indoc!("
                    According to all known laws of aviation, there is no way that a bee should be able to fly.
                    Its wings are too small to get its fat little body off the ground.

                    The bee, of course, flies anyway because bees don't care what humans think is impossible.
                "))
                .produces_ast("File[Par[[Word(According)|< >|Word(to)|< >|Word(all)|< >|Word(known)|< >|Word(laws)|< >|Word(of)|< >|Word(aviation,)|< >|Word(there)|< >|Word(is)|< >|Word(no)|< >|Word(way)|< >|Word(that)|< >|Word(a)|< >|Word(bee)|< >|Word(should)|< >|Word(be)|< >|Word(able)|< >|Word(to)|< >|Word(fly.)]|[Word(Its)|< >|Word(wings)|< >|Word(are)|< >|Word(too)|< >|Word(small)|< >|Word(to)|< >|Word(get)|< >|Word(its)|< >|Word(fat)|< >|Word(little)|< >|Word(body)|< >|Word(off)|< >|Word(the)|< >|Word(ground.)]]|Par[[Word(The)|< >|Word(bee,)|< >|Word(of)|< >|Word(course,)|< >|Word(flies)|< >|Word(anyway)|< >|Word(because)|< >|Word(bees)|< >|Word(don't)|< >|Word(care)|< >|Word(what)|< >|Word(humans)|< >|Word(think)|< >|Word(is)|< >|Word(impossible.)]]]"
            );
        }

        #[test]
        fn utf8() {
            ParserTest::new("cyrillic")
                .input("孫子 兵法")
                .produces_ast(r"File[Par[[Word(孫子)|< >|Word(兵法)]]]");
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
                let pluses = "+".repeat(num_pluses);
                let ast_pluses = ast_debug_pluses(num_pluses);

                ParserTest::new("command")
                    .input(format!(".order-66{pluses}"))
                    .produces_ast(format!("File[Par[[.order-66{ast_pluses}]]]"));
                ParserTest::new("with-qualifier")
                    .input(format!(".order.66{pluses}"))
                    .produces_ast(format!("File[Par[[.(order).66{ast_pluses}]]]"));
                ParserTest::new("trailing-dot")
                    .input(format!(".order.66{pluses}."))
                    .produces_ast(format!("File[Par[[.(order).66{ast_pluses}|Word(.)]]]"));

                ParserTest::new("many-qualifiers")
                    .input(format!(".it.belongs.in.a.museum{pluses}"))
                    .causes(format!("extra dots found at many-qualifiers[^:]*:1:12-12, many-qualifiers[^:]*:1:15-15, many-qualifiers[^:]*:1:17-17 in command name at many-qualifiers[^:]*:1:1-{}", 23 + num_pluses));
                ParserTest::new("empty-qualifier")
                    .input(format!("..what{pluses}"))
                    .causes(format!("empty qualifier found at empty-qualifier[^:]*:1:1-2 in command name at empty-qualifier[^:]*:1:1-{}", 6 + num_pluses));
            }
        }

        #[test]
        fn inline_args() {
            for num_pluses in 0..=3 {
                let pluses = "+".repeat(num_pluses);
                let ast_pluses = ast_debug_pluses(num_pluses);

                ParserTest::new("sole")
                    .input(format!(".exec{}{{order66}}", pluses))
                    .produces_ast(format!(
                        "File[Par[[.exec{}{{[Word(order66)]}}]]]",
                        ast_pluses
                    ));
                ParserTest::new("sole-with-qualifier")
                    .input(format!(".ex.ec{}{{order66}}", pluses))
                    .produces_ast(format!(
                        "File[Par[[.(ex).ec{}{{[Word(order66)]}}]]]",
                        ast_pluses
                    ));
                ParserTest::new("start of line")
                    .input(format!(".old-man-say{}{{leave her Johnny, leave her}} tomorrow ye will get your pay", pluses))
                    .produces_ast(format!("File[Par[[.old-man-say{}{{[Word(leave)|< >|Word(her)|< >|Word(Johnny,)|< >|Word(leave)|< >|Word(her)]}}|< >|Word(tomorrow)|< >|Word(ye)|< >|Word(will)|< >|Word(get)|< >|Word(your)|< >|Word(pay)]]]", ast_pluses));
                ParserTest::new("end of line")
                    .input(format!("I hate to .sail.on{}{{this rotten tub}}", pluses))
                    .produces_ast(format!("File[Par[[Word(I)|< >|Word(hate)|< >|Word(to)|< >|.(sail).on{}{{[Word(this)|< >|Word(rotten)|< >|Word(tub)]}}]]]", ast_pluses));
                ParserTest::new("middle of line")
                    .input(format!("For the .voyage-is{}{{foul}} and the winds don't blow", pluses))
                    .produces_ast(format!("File[Par[[Word(For)|< >|Word(the)|< >|.voyage-is{}{{[Word(foul)]}}|< >|Word(and)|< >|Word(the)|< >|Word(winds)|< >|Word(don't)|< >|Word(blow)]]]", ast_pluses));
                ParserTest::new("nested")
                    .input(format!(".no{}{{grog .allowed{}{{and}} rotten grub}}", pluses, pluses))
                    .produces_ast(format!("File[Par[[.no{}{{[Word(grog)|< >|.allowed{}{{[Word(and)]}}|< >|Word(rotten)|< >|Word(grub)]}}]]]", ast_pluses, ast_pluses));

                ParserTest::new("newline in brace-arg")
                    .input(format!(".order66{}{{\n}}", pluses))
                    .causes(format!(
                        "newline in braced args found at newline in brace-arg[^:]*:1:{}",
                        9 + num_pluses
                    ));
                ParserTest::new("newline in brace-arg")
                    .input(format!(".order66{}{{general\nkenobi}}", pluses))
                    .causes(format!(
                        "newline in braced args found at newline in brace-arg[^:]*:1:{}",
                        9 + num_pluses
                    ));
                ParserTest::new("par-break in brace-arg")
                    .input(format!(".order66{}{{\n\n}}", pluses))
                    .causes(format!(
                        "newline in braced args found at par-break in brace-arg[^:]*:1:{}",
                        9 + num_pluses
                    ));
                ParserTest::new("par-break in brace-arg")
                    .input(format!(".order66{}{{general\n\nkenobi}}", pluses))
                    .causes(format!(
                        "newline in braced args found at par-break in brace-arg[^:]*:1:{}",
                        9 + num_pluses
                    ));
                ParserTest::new("many-qualifiers")
                    .input(format!(".order.6.6{}{{general\n\nkenobi}}", pluses))
                    .causes(format!("extra dots found at many-qualifiers[^:]*:1:9-9 in command name at many-qualifiers[^:]*:1:1-{}", 10 + num_pluses));
            }
        }

        #[test]
        fn remainder_args() {
            for num_pluses in 0..=3 {
                let pluses = "+".repeat(num_pluses);
                let ast_pluses = ast_debug_pluses(num_pluses);

                ParserTest::new("many-qualifiers")
                    .input(format!(".qual.ifier{pluses}"))
                    .produces_ast(format!("File[Par[[.(qual).ifier{ast_pluses}]]]"));
                ParserTest::new("start of line")
                    .input(format!(".now{}{{we are ready}}: to sail for the horn", pluses))
                    .produces_ast(format!("File[Par[[.now{}{{[Word(we)|< >|Word(are)|< >|Word(ready)]}}:[Word(to)|< >|Word(sail)|< >|Word(for)|< >|Word(the)|< >|Word(horn)]]]]", ast_pluses));
                ParserTest::new("middle of line")
                    .input(format!("our boots .and{}{{our clothes boys}}, are all in the pawn", pluses))
                    .produces_ast(format!("File[Par[[Word(our)|< >|Word(boots)|< >|.and{}{{[Word(our)|< >|Word(clothes)|< >|Word(boys)]}}|Word(,)|< >|Word(are)|< >|Word(all)|< >|Word(in)|< >|Word(the)|< >|Word(pawn)]]]", ast_pluses));
                ParserTest::new("nested")
                    .input(format!("the anchor's on board .and{}{{the cable's}}: .all: stored", pluses))
                    .produces_ast(format!("File[Par[[Word(the)|< >|Word(anchor's)|< >|Word(on)|< >|Word(board)|< >|.and{}{{[Word(the)|< >|Word(cable's)]}}:[.all:[Word(stored)]]]]]", ast_pluses));
                ParserTest::new("nested in braces")
                    .input(format!("Heave away, bullies, .you{}{{parish-rigged bums, .take: your hands from your pockets and don’t}}: suck your thumbs", pluses))
                    .produces_ast(format!("File[Par[[Word(Heave)|< >|Word(away,)|< >|Word(bullies,)|< >|.you{}{{[Word(parish)|-|Word(rigged)|< >|Word(bums,)|< >|.take:[Word(your)|< >|Word(hands)|< >|Word(from)|< >|Word(your)|< >|Word(pockets)|< >|Word(and)|< >|Word(don’t)]]}}:[Word(suck)|< >|Word(your)|< >|Word(thumbs)]]]]", ast_pluses));
                ParserTest::new("stacked")
                    .input(format!(".heave{}{{a pawl}}:, o heave away\n.way{}{{hay}}: roll 'an go!", pluses, pluses))
                    .produces_ast(format!("File[Par[[.heave{}{{[Word(a)|< >|Word(pawl)]}}:[Word(,)|< >|Word(o)|< >|Word(heave)|< >|Word(away)]]|[.way{}{{[Word(hay)]}}:[Word(roll)|< >|Word('an)|< >|Word(go!)]]]]", ast_pluses, ast_pluses));

                ParserTest::new("sole at end of line")
                    .input(".randy-dandy-o:")
                    .causes("Unrecognised EOF found at (1:16|2:1)");
                ParserTest::new("end of line")
                    .input("randy .dandy-o:")
                    .causes("Unrecognised token `newline` found at 1:1[56]");
                ParserTest::new("many-qualifiers")
                    .input(".randy.dandy.o")
                    .causes("extra dots found at many-qualifiers[^:]*:1:13-13 in command name at many-qualifiers[^:]*:1:1-14");
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
                        name: "command-with-qualifier",
                        data: &[
                            &format!(".come.all{}{{you}}:", pluses),
                            "\tyoung sailor men",
                            "\tlisten to me",
                            "::",
                            "\tI'll sing you a song",
                            "\tof the fish in the sea",
                        ],
                        expected_structure: &format!("File[Par[.(come).all{}{{[Word(you)]}}::[Par[[Word(young)|< >|Word(sailor)|< >|Word(men)]|[Word(listen)|< >|Word(to)|< >|Word(me)]]]::[Par[[Word(I'll)|< >|Word(sing)|< >|Word(you)|< >|Word(a)|< >|Word(song)]|[Word(of)|< >|Word(the)|< >|Word(fish)|< >|Word(in)|< >|Word(the)|< >|Word(sea)]]]]]", ast_pluses),
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
                    ParserTest::new(name_with_tabs)
                        .input(data_with_tabs)
                        .produces_ast(test.expected_structure);

                    let name_with_spaces = format!("{} (with spaces)", test.name);
                    let data_with_spaces = test
                        .data
                        .iter()
                        .map(|l| l.replace('\t', "    "))
                        .collect::<Vec<_>>()
                        .join("\n");
                    ParserTest::new(name_with_spaces)
                        .input(data_with_spaces)
                        .produces_ast(test.expected_structure);
                }

                ParserTest::new("end of populated line")
                    .input(indoc::indoc!(
                        "
                            william taylor was a .brisk{young sailor}:
                            \tfull of heart and full of play
                        "
                    ))
                    .causes("Unrecognised token `newline` found at 1:43:2:1");
                ParserTest::new("missing indent")
                    .input(indoc::indoc!(
                        "
                            .until{did his mind uncover}:
                            to a youthful lady gay
                        "
                    ))
                    .causes("Unrecognised token `word` found at 2:1:2:3");
                ParserTest::new("missing second trailer")
                    .input(indoc::indoc!(
                        "
                            .until{did his mind uncover}:
                            \tto a youthful lady gay
                            ::
                            \tfour and twenty british sailors
                            ::
                        "
                    ))
                    .causes("Unrecognised EOF found at (5:3|6:1)");
                ParserTest::new("many-qualifiers")
                    .input(indoc::indoc!(
                        "
                            .met.him.on.the{king's highway}:
                            \tas he went for to be married
                            ::
                            \tpressed he was and sent away
                        "
                    ))
                    .causes("extra dots found at many-qualifiers[^:]*:1:9-9, many-qualifiers[^:]*:1:12-12 in command name at many-qualifiers[^:]*:1:1-15");
            }
        }

        #[test]
        fn attrs() {
            ParserTest::new("empty")
                .input("we are .outward-bound[]")
                .produces_ast("File[Par[[Word(we)|< >|Word(are)|< >|.outward-bound[]]]]");
            ParserTest::new("unnamed-only")
                .input("we are .outward-bound[for,kingston,town]")
                .produces_ast("File[Par[[Word(we)|< >|Word(are)|< >|.outward-bound[(for)|(kingston)|(town)]]]]");
            ParserTest::new("unnamed-only-with-spaces")
                .input("we are .outward-bound[ for , kingston , town ]")
                .produces_ast("File[Par[[Word(we)|< >|Word(are)|< >|.outward-bound[(for)|(kingston)|(town)]]]]");
            ParserTest::new("unnamed-only-with-tabs")
                .input("we are .outward-bound[\tfor\t,\tkingston\t,\ttown\t]")
                .produces_ast(r"File[Par[[Word(we)|< >|Word(are)|< >|.outward-bound[(for)|(kingston)|(town)]]]]");

            ParserTest::new("named")
                .input("we are .outward-bound[for=kingston,town]")
                .produces_ast("File[Par[[Word(we)|< >|Word(are)|< >|.outward-bound[(for)=(kingston)|(town)]]]]");
            ParserTest::new("named-with-spaces")
                .input("we are .outward-bound[   for   =   kingston   ,   town   ]")
                .produces_ast("File[Par[[Word(we)|< >|Word(are)|< >|.outward-bound[(for)=(kingston)|(town)]]]]");
            ParserTest::new("named-with-spaces")
                .input("we are .outward-bound[\tfor\t=\tkingston\t,\ttown\t]")
                .produces_ast(r"File[Par[[Word(we)|< >|Word(are)|< >|.outward-bound[(for)=(kingston)|(town)]]]]");

            ParserTest::new("with-inline-args")
                .input("and we'll .heave[the,old=wheel,round]{and}{round}")
                .produces_ast(r"File[Par[[Word(and)|< >|Word(we'll)|< >|.heave[(the)|(old)=(wheel)|(round)]{[Word(and)]}{[Word(round)]}]]]");
            ParserTest::new("with-trailer-args")
                .input("and we'll .heave[the,old=wheel,round]: and round")
                .produces_ast(r"File[Par[[Word(and)|< >|Word(we'll)|< >|.heave[(the)|(old)=(wheel)|(round)]:[Word(and)|< >|Word(round)]]]]");
            ParserTest::new("with-remainder-args")
                .input(".heave[the,old=wheel,round]:\n\tand\n::\n\tround")
                .produces_ast(r"File[Par[.heave[(the)|(old)=(wheel)|(round)]::[Par[[Word(and)]]]::[Par[[Word(round)]]]]]");

            ParserTest::new("unclosed")
                .input(".heave[")
                .causes("(unexpected EOF found at 1:8|newline in attributes found at unclosed with newline:1:7-7)");
            ParserTest::new("unexpected open bracket")
                .input(".heave[[")
                .causes(r"(unexpected EOF found at 1:9|unexpected character '\[' found at unexpected open bracket[^:]*:1:8-8)");
        }
    }

    mod interword {
        use std::fmt::Display;

        use super::*;

        fn test_dash<T>(name: T, dash: &'static str, repr: &'static str)
        where
            T: Into<Cow<'static, str>> + Clone,
        {
            ParserTest::new(name.clone())
                .input(dash)
                .produces_ast(format!("File[Par[[{}]]]", repr));
            ParserTest::new(name.clone())
                .input(format!("a{}b", dash))
                .produces_ast(format!("File[Par[[Word(a)|{}|Word(b)]]]", repr));
            ParserTest::new(name.clone())
                .input(format!("a {}b", dash))
                .produces_ast(format!("File[Par[[Word(a)|< >|{}|Word(b)]]]", repr));
            ParserTest::new(name.clone())
                .input(format!("a{} b", dash))
                .produces_ast(format!("File[Par[[Word(a)|{}|< >|Word(b)]]]", repr));
            ParserTest::new(name.clone())
                .input(format!("a {} b", dash))
                .produces_ast(format!("File[Par[[Word(a)|< >|{}|< >|Word(b)]]]", repr));
            ParserTest::new(name.clone())
                .input(format!("a\n{}b", dash))
                .produces_ast(format!("File[Par[[Word(a)]|[{}|Word(b)]]]", repr));
            ParserTest::new(name)
                .input(format!("a{}\nb", dash))
                .produces_ast(format!("File[Par[[Word(a)|{}]|[Word(b)]]]", repr));
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

        fn test_glue<T>(name: T, glue: &'static str, repr: &'static str)
        where
            T: Into<Cow<'static, str>> + Clone,
        {
            ParserTest::new(name.clone())
                .input(glue)
                .produces_ast(format!("File[Par[[SpiltGlue({})]]]", repr));
            ParserTest::new(name.clone())
                .input(format!("a{}b", glue))
                .produces_ast(format!("File[Par[[Word(a)|{}|Word(b)]]]", repr));
            ParserTest::new(name.clone())
                .input(format!("a {}b", glue))
                .produces_ast(format!("File[Par[[Word(a)|SpiltGlue( {})|Word(b)]]]", repr));
            ParserTest::new(name.clone())
                .input(format!("a{} b", glue))
                .produces_ast(format!("File[Par[[Word(a)|SpiltGlue({} )|Word(b)]]]", repr));
            ParserTest::new(name.clone())
                .input(format!("a {} b", glue))
                .produces_ast(format!(
                    "File[Par[[Word(a)|SpiltGlue( {} )|Word(b)]]]",
                    repr
                ));
            ParserTest::new(name.clone())
                .input(format!("a\n{}b", glue))
                .produces_ast(format!(
                    "File[Par[[Word(a)]|[SpiltGlue({})|Word(b)]]]",
                    repr
                ));
            ParserTest::new(name.clone())
                .input(format!("a{}\nb", glue))
                .produces_ast(format!(
                    "File[Par[[Word(a)|SpiltGlue({})]|[Word(b)]]]",
                    repr
                ));
            ParserTest::new(name.clone())
                .input(format!("a{}", glue))
                .produces_ast(format!("File[Par[[Word(a)|SpiltGlue({})]]]", repr));
            ParserTest::new(name)
                .input(format!("{}b", glue))
                .produces_ast(format!("File[Par[[SpiltGlue({})|Word(b)]]]", repr));
        }

        #[test]
        fn glue() {
            test_glue("glue", "~", "~");
        }

        #[test]
        fn nbsp() {
            test_glue("nbsp", "~~", "~~");
        }

        #[test]
        fn mixed() {
            test_dash("em-hyph", "----", "---|-");
            test_dash("em-en", "-----", "---|--");
            test_dash("em-em", "------", "---|---");

            fn test_mix<S, T, R>(name: S, to_test: T, repr: R)
            where
                S: Into<Cow<'static, str>> + Clone,
                T: Into<Cow<'static, str>> + Clone + Display,
                R: Into<Cow<'static, str>> + Clone + Display,
            {
                ParserTest::new(name)
                    .input(format!("a{to_test}b"))
                    .produces_ast(format!("File[Par[[Word(a)|{repr}|Word(b)]]]"));
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
                    format!("{raw}-",),
                    format!("{repr}|-"),
                );
                test_mix(
                    "glue-mixed-1-dash-2",
                    format!("-{raw}",),
                    format!("-|{repr}"),
                );
                test_mix(
                    "glue-mixed-2-dashes-1",
                    format!("{raw}--",),
                    format!("{repr}|--"),
                );
                test_mix(
                    "glue-mixed-2-dashes-2",
                    format!("-{raw}-",),
                    format!("-|{repr}|-"),
                );
                test_mix(
                    "glue-mixed-2-dashes-3",
                    format!("--{raw}",),
                    format!("--|{repr}"),
                );
                test_mix(
                    "glue-mixed-3-dashes-1",
                    format!("{raw}---",),
                    format!("{repr}|---"),
                );
                test_mix(
                    "glue-mixed-3-dashes-2",
                    format!("-{raw}--",),
                    format!("-|{repr}|--"),
                );
                test_mix(
                    "glue-mixed-3-dashes-3",
                    format!("--{raw}-",),
                    format!("--|{repr}|-"),
                );
                test_mix(
                    "glue-mixed-3-dashes-4",
                    format!("---{raw}",),
                    format!("---|{repr}"),
                );
            }
        }
    }

    mod verbatim {
        use super::*;

        #[test]
        fn word() {
            ParserTest::new("ignore empty")
                .input("!!")
                .produces_ast("File[Par[[Word(!!)]]]");
            ParserTest::new("ignore unmatched at start")
                .input("spanish inquisition!")
                .produces_ast("File[Par[[Word(spanish)|< >|Word(inquisition!)]]]");
            ParserTest::new("ignore unmatched at end")
                .input("!spanish inquisition")
                .produces_ast("File[Par[[Word(!spanish)|< >|Word(inquisition)]]]");
        }

        #[test]
        fn short() {
            ParserTest::new("text")
                .input("!verb!")
                .produces_ast("File[Par[[!verb!]]]");
            ParserTest::new("comment")
                .input("!//!")
                .produces_ast("File[Par[[!//!]]]");
            ParserTest::new("multi line comment start")
                .input("!/*!")
                .produces_ast("File[Par[[!/*!]]]");
            ParserTest::new("multi line comment end")
                .input("!*/!")
                .produces_ast("File[Par[[!*/!]]]");
            ParserTest::new("with spaces")
                .input("!hello } world :: !")
                .produces_ast("File[Par[[!hello } world :: !]]]");
            ParserTest::new("ignored in comment")
                .input("//!asdf!")
                .produces_ast("File[Par[[//!asdf!]]]");
        }

        #[test]
        fn multiple() {
            ParserTest::new("multiple-single-line")
                .input("!verb1! !verb2!")
                .produces_ast("File[Par[[!verb1!|< >|!verb2!]]]");
            ParserTest::new("multiple-single-line")
                .input("!verb1!\n!verb2!")
                .produces_ast("File[Par[[!verb1!]|[!verb2!]]]");
        }
    }

    mod single_line_comments {
        use super::*;

        #[test]
        fn whole_line() {
            ParserTest::new("sole")
                .input("//")
                .produces_ast("File[Par[[//]]]");
            ParserTest::new("no gap")
                .input("//hello, world!\n")
                .produces_ast("File[Par[[//hello, world!]]]");
            ParserTest::new("leading space")
                .input("// hello, world!\n")
                .produces_ast("File[Par[[// hello, world!]]]");
            ParserTest::new("leading tab")
                .input("//\thello, world!\n")
                .produces_ast(r"File[Par[[//\thello, world!]]]");
        }

        #[test]
        fn partial() {
            ParserTest::new("no gap")
                .input("to me!//to you!\n")
                .produces_ast(r"File[Par[[Word(to)|< >|Word(me!)|//to you!]]]");
            ParserTest::new("space-after-comment")
                .input("to me!// to you!\n")
                .produces_ast(r"File[Par[[Word(to)|< >|Word(me!)|// to you!]]]");
            ParserTest::new("space-after-comment")
                .input("to me!//\tto you!\n")
                .produces_ast(r"File[Par[[Word(to)|< >|Word(me!)|//\tto you!]]]");
            ParserTest::new("space-before-comment")
                .input("to me! //to you!\n")
                .produces_ast("File[Par[[Word(to)|< >|Word(me!)|< >|//to you!]]]");
            ParserTest::new("tab-before-comment")
                .input("to me!\t//to you!\n")
                .produces_ast(r"File[Par[[Word(to)|< >|Word(me!)|<\t>|//to you!]]]");
            ParserTest::new("surrounding-spaces")
                .input("to me! // to you!\n")
                .produces_ast(r"File[Par[[Word(to)|< >|Word(me!)|< >|// to you!]]]");
            ParserTest::new("surrounding-tabs")
                .input("to me!\t//\tto you!\n")
                .produces_ast(r"File[Par[[Word(to)|< >|Word(me!)|<\t>|//\tto you!]]]");
            ParserTest::new("surrounding-mix-1")
                .input("to me! //\tto you!\n")
                .produces_ast(r"File[Par[[Word(to)|< >|Word(me!)|< >|//\tto you!]]]");
            ParserTest::new("surrounding-mix-2")
                .input("to me!\t// to you!\n")
                .produces_ast(r"File[Par[[Word(to)|< >|Word(me!)|<\t>|// to you!]]]");
        }

        #[test]
        fn stacked() {
            let lines = [
                "There once was a ship that put to sea",
                "And the name of that ship was the Billy O’ Tea",
                "The winds blew hard, her bow dipped down",
                "Blow, me bully boys, blow",
            ];
            ParserTest::new("multiple-comments")
                .input(format!("//{}\n", lines.join("\n//")))
                .produces_ast(format!(
                    "File[Par[[{}]]]",
                    lines
                        .iter()
                        .map(|l| format!("//{}", l))
                        .collect::<Vec<_>>()
                        .join("]|[")
                ));
        }

        #[test]
        fn as_trailer_arg() {
            ParserTest::new("comment as sole arg")
                .input(".spaghetti:\n\t//and meatballs")
                .produces_ast("File[Par[.spaghetti::[Par[[//and meatballs]]]]]");
        }
    }

    mod multi_line_comments {
        use super::*;

        #[test]
        fn empty() {
            ParserTest::new("single")
                .input("/**/")
                .produces_ast(r"File[Par[[/*[]*/]]]");
            ParserTest::new("multiple")
                .input("/**//**/")
                .produces_ast(r"File[Par[[/*[]*/|/*[]*/]]]");
            ParserTest::new("multiple with space gap")
                .input("/**/ /**/")
                .produces_ast(r"File[Par[[/*[]*/|< >|/*[]*/]]]");
            ParserTest::new("multiple with tab gap")
                .input("/**/\t/**/")
                .produces_ast(r"File[Par[[/*[]*/|<\t>|/*[]*/]]]");
            ParserTest::new("stacked")
                .input(indoc::indoc!(
                    r"
                        /**/
                        /**/
                    "
                ))
                .produces_ast(r"File[Par[[/*[]*/]|[/*[]*/]]]");
            ParserTest::new("pars with stacked")
                .input("/**/\n\n/**/\n/**/")
                .produces_ast(r"File[Par[[/*[]*/]]|Par[[/*[]*/]|[/*[]*/]]]");
        }

        #[test]
        fn whitespace_contents() {
            ParserTest::new("space only")
                .input("/* */")
                .produces_ast(r"File[Par[[/*[ ]*/]]]");
            ParserTest::new("tab only")
                .input("/*\t*/")
                .produces_ast(r"File[Par[[/*[\t]*/]]]");
        }

        #[test]
        fn with_text() {
            ParserTest::new("text")
                .input("/*spaghetti and meatballs*/")
                .produces_ast(r"File[Par[[/*[spaghetti and meatballs]*/]]]");
            ParserTest::new("text with surrounding space")
                .input("/* spaghetti and meatballs */")
                .produces_ast(r"File[Par[[/*[ spaghetti and meatballs ]*/]]]");
            ParserTest::new("text with newline")
                .input("/*spaghetti and\nmeatballs*/")
                .produces_ast(r"File[Par[[/*[spaghetti and|\n|meatballs]*/]]]");
            ParserTest::new("multiple comments")
                .input("/*spaghetti*/\n/*and*/\n\n/*meatballs*/")
                .produces_ast(r"File[Par[[/*[spaghetti]*/]|[/*[and]*/]]|Par[[/*[meatballs]*/]]]");
        }

        #[test]
        fn nested() {
            ParserTest::new("nested comment")
                .input("/*spaghetti/*and*/meatballs*/")
                .produces_ast(r"File[Par[[/*[spaghetti|Nested[and]|meatballs]*/]]]");
            ParserTest::new("nested and indented comment")
                .input("/*spaghetti\n\t/*\n\t\tand\n\t*/\nmeatballs*/")
                .produces_ast(
                    r"File[Par[[/*[spaghetti|\n|\t|Nested[\n|\t\tand|\n|\t]|\n|meatballs]*/]]]",
                );
            ParserTest::new("nested unindented comment")
                .input("/*spaghetti\n\t/*\nand\n\t*/\nmeatballs*/")
                .produces_ast(
                    r"File[Par[[/*[spaghetti|\n|\t|Nested[\n|and|\n|\t]|\n|meatballs]*/]]]",
                );
        }

        #[test]
        fn unmatched() {
            ParserTest::new("open")
                .input("/*spaghetti/*and*/meatballs")
                .causes(r#"unclosed comment found at \["open[^:]*:1:1-2"\]"#);
            ParserTest::new("open")
                .input("/*spaghetti/*and meatballs")
                .causes(r#"unclosed comment found at \["open[^:]*:1:1-2", "open[^:]*:1:12-13"\]"#);
            ParserTest::new("close")
                .input("spaghetti/*and*/meatballs */")
                .causes("no comment to close found at close[^:]*:1:27-28");
        }

        #[test]
        fn as_trailer_arg() {
            ParserTest::new("comment as sole arg")
                .input(".spaghetti:\n\t/*and meatballs*/")
                .produces_ast("File[Par[.spaghetti::[Par[[/*[and meatballs]*/]]]]]");
        }

        #[test]
        fn final_indentation() {
            ParserTest::new("final tab indent")
                .input("/*spaghetti\n\t*/")
                .produces_ast(r"File[Par[[/*[spaghetti|\n|\t]*/]]]");
            ParserTest::new("final spaces indent")
                .input("/*spaghetti\n    */")
                .produces_ast(r"File[Par[[/*[spaghetti|\n|    ]*/]]]");
            ParserTest::new("long, prettified comment block")
                .input("/* spaghetti\n *and\n *meatballs\n */")
                .produces_ast(r"File[Par[[/*[ spaghetti|\n| *and|\n| *meatballs|\n| ]*/]]]");
        }

        #[test]
        fn before_remainder_args() {
            ParserTest::new("single-line")
                .input("/*spaghetti*/.and: meatballs")
                .produces_ast("File[Par[[/*[spaghetti]*/|.and:[Word(meatballs)]]]]");
            ParserTest::new("multi-line")
                .input("/*spaghetti\n\t\t*/.and: meatballs")
                .produces_ast(r"File[Par[[/*[spaghetti|\n|\t\t]*/|.and:[Word(meatballs)]]]]");
        }

        #[test]
        fn before_trailer_args() {
            ParserTest::new("trailer-args")
                .input("/*spaghetti*/.and:\n\tmeatballs")
                .causes("Unrecognised token `newline` found at 1:19");
        }
    }

    mod syntactic_sugar {
        use super::*;

        #[test]
        fn mark() {
            ParserTest::new("sole")
                .input("@foo")
                .produces_ast("File[Par[[$mark[foo]]]]");
            ParserTest::new("mid-line")
                .input("hello @sup world")
                .produces_ast(r"File[Par[[Word(hello)|< >|$mark[sup]|< >|Word(world)]]]");
            ParserTest::new("in-heading")
                .input("# @asdf")
                .produces_ast(r"File[Par[[$h1{[$mark[asdf]]}]]]");
            for c in ['!', '?', '\'', '"', '(', ')'] {
                let repr = match c {
                    '"' | '(' | ')' => format!(r"\{c}"),
                    c => c.into(),
                };
                ParserTest::new(format!("with-terminator-{c}"))
                    .input(format!("#foo{c}"))
                    .produces_ast(format!("File[Par[[$ref[foo]|Word({repr})]]]"));
            }
        }

        #[test]
        fn reference() {
            ParserTest::new("sole")
                .input("#foo")
                .produces_ast("File[Par[[$ref[foo]]]]");
            ParserTest::new("mid-line")
                .input("hello #world!")
                .produces_ast("File[Par[[Word(hello)|< >|$ref[world]|Word(!)]]]");
            ParserTest::new("in-heading")
                .input("# #foo")
                .produces_ast("File[Par[[$h1{[$ref[foo]]}]]]");
            for c in ['!', '?', '\'', '"', '(', ')'] {
                let repr = match c {
                    '"' | '(' | ')' => format!(r"\{c}"),
                    c => c.into(),
                };
                ParserTest::new(format!("with-terminator-{c}"))
                    .input(format!("#foo{c}"))
                    .produces_ast(format!("File[Par[[$ref[foo]|Word({repr})]]]"));
            }
        }

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
                        ParserTest::new(format!("normal nesting {outer} and {inner}"))
                            .input(format!("{outer}spaghetti {inner}and{inner} meatballs{outer}"))
                            .produces_ast(format!("File[Par[[{outer_repr}{{[Word(spaghetti)|< >|{inner_repr}{{[Word(and)]}}|< >|Word(meatballs)]}}]]]"));

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
                            let sanitised_inner = inner.replace('*', r"\*");
                            ParserTest::new(format!("clash-nesting {inner} and {outer}"))
                                .input(format!("{outer}spaghetti {inner}and meatballs{inner}{outer}"))
                                .causes(format!(r"delimiter mismatch for {sanitised_inner} found at clash-nesting {sanitised_inner} and {sanitised_outer}[^:]*:1:{}-{} \(failed to match at clash-nesting {sanitised_inner} and {sanitised_outer}[^:]*:1:{}-{}\)", 24 + outer.len() + inner.len(), 24 + outer.len() + 2 * inner.len(), 11 + outer.len(), 10 + outer.len() + inner.len()));
                        } else {
                            // Check nesting is okay
                            ParserTest::new(format!("chash-nesting nesting {outer} and meatballs{inner}"))
                                .input(format!("{outer}spaghetti {inner}and meatballs{inner}{outer}"))
                                .produces_ast(format!("File[Par[[{outer_repr}{{[Word(spaghetti)|< >|{inner_repr}{{[Word(and)|< >|Word(meatballs)]}}]}}]]]"));
                        }
                    }
                }
            }

            #[test]
            fn multi_line() {
                for (delim, _) in &DELIMS {
                    let sanitised = delim.replace('*', r"\*");
                    ParserTest::new(format!("multi-line {delim}"))
                        .input(format!("{delim}foo\nbar{delim}"))
                        .causes(format!(r#"newline in "{sanitised}" emphasis found at multi-line {sanitised}[^:]*:1:{}-2:1"#, 4 + delim.len()));
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
                        ParserTest::new(format!("mismatch({left},{right})"))
                            .input(format!("{left}foo{right}"))
                            .causes(format!(r"delimiter mismatch for {sanitised_left} found at mismatch\({sanitised_left},{sanitised_right}\)[^:]*:1:{}-{} \(failed to match at mismatch\({sanitised_left},{sanitised_right}\)[^:]*:1:1-{}\)", 4 + left.len(), 3 + left.len() + right.len(), left.len(),));
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
                        ParserTest::new(format!("level:{level}, pluses:{pluses}"))
                            .input(format!("{}{} foo", "#".repeat(level), "+".repeat(pluses)))
                            .produces_ast(format!(
                                "File[Par[[$h{level}{}{{[Word(foo)]}}]]]",
                                if pluses > 0 {
                                    format!("({})", "+".repeat(pluses))
                                } else {
                                    "".into()
                                }
                            ));
                    }
                }

                ParserTest::new("nested inline args")
                    .input("## .bar{baz}")
                    .produces_ast("File[Par[[$h2{[.bar{[Word(baz)]}]}]]]");
                ParserTest::new("nested remainder args")
                    .input("## .bar: baz")
                    .produces_ast("File[Par[[$h2{[.bar:[Word(baz)]]}]]]");
                ParserTest::new("nested trailer args")
                    .input("## .foo:\n\tbar")
                    .causes("Unrecognised token `newline` found at 1:9:2:1");

                ParserTest::new("nested headings")
                    .input("## ## foo")
                    .causes("unexpected heading at nested headings[^:]*:1:4-5");
                ParserTest::new("no argument")
                    .input("##")
                    .causes("Unrecognised token `newline` found at (1:1:1:3|1:3:2:1)");
            }

            #[test]
            fn midline() {
                ParserTest::new("inline")
                    .input("foo ###+ bar")
                    .causes("unexpected heading at inline[^:]*:1:5-8");
                ParserTest::new("inline-complex")
                    .input("foo .bar: ###+ baz")
                    .causes("unexpected heading at inline[^:]*:1:11-14");
            }

            #[test]
            fn too_deep() {
                ParserTest::new("plain")
                    .input("#######")
                    .causes(r"heading too deep at plain[^:]*:1:1-7 \(7 levels\)");
                ParserTest::new("with-plus")
                    .input("#######+")
                    .causes(r"heading too deep at with-plus[^:]*:1:1-8 \(7 levels\)");
            }
        }
    }
}
