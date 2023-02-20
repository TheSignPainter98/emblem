use crate::log::messages::{
    DelimiterMismatch, ExtraCommentClose, NewlineInEmphDelimiter, NewlineInInlineArg,
    UnclosedComments, UnexpectedChar, UnexpectedEOF,
};
use crate::log::Log;
use crate::parser::Location;
use crate::{log::messages::Message, parser::point::Point};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::VecDeque,
    error::Error,
    fmt::{self, Display},
};

pub struct Lexer<'input> {
    input: &'input str,
    done: bool,
    start_of_line: bool,
    current_indent: u32,
    curr_point: Point<'input>,
    prev_point: Point<'input>,
    open_braces: Vec<Location<'input>>,
    next_toks: VecDeque<SpannedTok<'input>>,
    multi_line_comment_starts: Vec<Location<'input>>,
    last_tok: Option<Tok<'input>>,
    parsing_attrs: bool,
    opening_delimiters: bool,
    open_delimiters: Vec<(&'input str, Location<'input>)>,
}

impl<'input> Lexer<'input> {
    pub fn new(file: &'input str, input: &'input str) -> Self {
        Self {
            input,
            done: false,
            start_of_line: true,
            current_indent: 0,
            curr_point: Point::new(file, input),
            prev_point: Point::new(file, input),
            open_braces: Vec::new(),
            next_toks: VecDeque::new(),
            multi_line_comment_starts: Vec::new(),
            last_tok: None,
            parsing_attrs: false,
            opening_delimiters: true,
            open_delimiters: Vec::new(),
        }
    }

    fn try_consume(&mut self, re: &Regex) -> Option<&'input str> {
        if let Some(mat) = re.find(self.input) {
            self.input = &self.input[mat.end()..];

            let curr_point = self.curr_point.clone();
            self.prev_point = curr_point.clone();
            self.curr_point = curr_point.shift(mat.as_str());

            Some(mat.as_str())
        } else {
            None
        }
    }

    fn span(&self, tok: Tok<'input>) -> SpannedTok<'input> {
        (self.prev_point.clone(), tok, self.curr_point.clone())
    }

    fn enqueue_indentation_level(&mut self, target: u32) {
        let difference = self.current_indent.abs_diff(target);

        if difference == 0 {
            return;
        }

        let tok = if self.current_indent < target {
            Tok::Indent
        } else {
            Tok::Dedent
        };

        for _ in 0..difference {
            self.enqueue(self.span(tok.clone()))
        }

        self.current_indent = target;
    }

    fn dequeue(&mut self) -> Option<SpannedTok<'input>> {
        self.next_toks.pop_front()
    }

    fn enqueue(&mut self, t: SpannedTok<'input>) {
        self.next_toks.push_back(t)
    }

    fn can_start_attrs(&self) -> bool {
        matches!(self.last_tok, Some(Tok::Command(_, _)))
    }

    fn location(&self) -> Location<'input> {
        Location::new(&self.prev_point, &self.curr_point)
    }

    fn emph(&mut self, raw: &'input str) -> Result<Tok<'input>, Box<LexicalError<'input>>> {
        if self.opening_delimiters {
            self.open_delimiters.push((raw, self.location()));

            return match raw {
                "_" | "*" => Ok(Tok::ItalicOpen(raw)),
                "__" | "**" => Ok(Tok::BoldOpen(raw)),
                "=" => Ok(Tok::SmallcapsOpen(raw)),
                "==" => Ok(Tok::AlternateFaceOpen(raw)),
                "`" => Ok(Tok::MonospaceOpen(raw)),
                _ => panic!("internal error: unknown emphasis string {:?}", raw),
            };
        }

        if !self.open_delimiters.is_empty() {
            let (to_close, to_close_loc) = self.open_delimiters.pop().unwrap();
            if to_close != raw {
                return Err(Box::new(LexicalError::DelimiterMismatch {
                    loc: self.location(),
                    to_close_loc,
                    expected: to_close,
                }));
            }
        }

        match raw {
            "_" | "*" => Ok(Tok::ItalicClose),
            "__" | "**" => Ok(Tok::BoldClose),
            "=" => Ok(Tok::SmallcapsClose),
            "==" => Ok(Tok::AlternateFaceClose),
            "`" => Ok(Tok::MonospaceClose),
            _ => panic!("internal error: unknown emphasis string {:?}", raw),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<SpannedTok<'input>, Box<LexicalError<'input>>>;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! token_patterns {
            ( $(let $name:ident = $pattern:literal);* $(;)? ) => {
                lazy_static! {
                    $(static ref $name: Regex = Regex::new(concat!("^", $pattern)).unwrap();)*
                }
            }
        }

        token_patterns! {
            let WORD           = r"([^ /\t\r\n}_*`=~-]|/[^ /\t\r\n}_*`=~-])+";
            let WHITESPACE     = r"[ \t]+";
            let PAR_BREAKS     = r"([ \t]*(\n|\r\n|\r))+";
            let LN             = r"(\n|\r\n|\r)";
            let COLON          = r":[ \t]*";
            let DOUBLE_COLON   = r"::";
            let INITIAL_INDENT = r"[ \t]*";
            let COMMAND        = r"\.[^ \t{}\[\]\r\n:+]+\+*";
            let VERBATIM       = r"![^\r\n]*!";
            let BRACE_LEFT     = r"\{";
            let BRACE_RIGHT    = r"\}";
            let COMMENT        = r"//[^\r\n]*";
            let DASH           = r"-{1,3}";
            let GLUE           = r"~~?";
            let UNDERSCORES    = r"_{1,2}";
            let ASTERISKS      = r"\*{1,2}";
            let EQUALS         = r"={1,2}";
            let BACKTICKS      = r"`";

            let OPEN_ATTRS   = r"\[";
            let CLOSE_ATTRS  = r"]";
            let COMMA        = r",";
            let UNNAMED_ATTR = r"[ \t]*([^,= \t\[\]]|\\[,=\[\]])+[ \t]*";
            let NAMED_ATTR   = r"[ \t]*([^,= \t\[\]]|\\[,=\[\]])+[ \t]*=[ \t]*([^,\[\]]|\\[,\[\]])*[ \t]*";

            let NESTED_COMMENT_OPEN  = r"/\*";
            let NESTED_COMMENT_CLOSE = r"\*/";
            let NESTED_COMMENT_PART  = r"([^*/\r\n]|\*[^/\r\n]|/[^*\r\n])+";
        }

        if let Some(t) = self.dequeue() {
            return Some(Ok(t));
        }

        if self.done {
            return None;
        }

        macro_rules! match_token {
            ( ! => $on_eof:expr, $($re:ident => $to_tok:expr),* $(,)? ) => {
                if self.input.is_empty() {
                    #[allow(unreachable_code)]
                    Some($on_eof)
                }
                $(else if let Some(mat) = self.try_consume(&$re) {
                    let ret = $to_tok(mat).map(|t| self.span(t));
                    self.last_tok = ret.as_ref().ok().map(|s| s.1.clone());
                    Some(ret)
                })*
                else {
                    Some(Err(Box::new(
                        LexicalError::UnexpectedChar {
                            found: self.input.chars().next().unwrap(),
                            loc: self.location(),
                        }
                    )))
                }
            };
        }

        if !self.multi_line_comment_starts.is_empty() {
            return match_token![
                ! => {
                    Err(Box::new(LexicalError::UnmatchedCommentOpen {
                        unclosed: self.multi_line_comment_starts.clone(),
                    }))
                },

                NESTED_COMMENT_PART => |s: &'input str| Ok(Tok::Comment(s)) ,
                LN                  => |_| Ok(Tok::Newline) ,
                NESTED_COMMENT_OPEN => |_| {
                    self.multi_line_comment_starts.push(self.location());
                    Ok(Tok::NestedCommentOpen)
                },
                NESTED_COMMENT_CLOSE => |_| {
                    self.multi_line_comment_starts.pop();
                    Ok(Tok::NestedCommentClose)
                },
            ];
        }

        if self.parsing_attrs {
            return match_token! {
                ! => {
                    self.done = true;
                    Err(Box::new(LexicalError::UnexpectedEOF {
                        point: self.curr_point.clone(),
                        expected: vec![],
                    }))
                },

                NAMED_ATTR   => |s: &'input str| Ok(Tok::NamedAttr(s)),
                UNNAMED_ATTR => |s: &'input str| Ok(Tok::UnnamedAttr(s)),
                COMMA        => |_| Ok(Tok::AttrComma),
                CLOSE_ATTRS  => |_| {
                    self.parsing_attrs = false;
                    Ok(Tok::RBracket)
                },
            };
        }

        if self.input.is_empty() {
            self.done = true;
            if !self.open_braces.is_empty() {
                return Some(Err(Box::new(LexicalError::UnexpectedEOF {
                    point: self.curr_point.clone(),
                    expected: vec!["\"}\"".into()],
                })));
            }

            if self.last_tok != Some(Tok::Newline) {
                self.enqueue(self.span(Tok::Newline));
            }
            self.enqueue_indentation_level(0);
            return self.dequeue().map(Ok);
        }

        if self.try_consume(&LN).is_some() {
            self.start_of_line = true;
            self.opening_delimiters = true;

            if !self.open_braces.is_empty() {
                return Some(Err(Box::new(LexicalError::NewlineInArg {
                    arg_start_loc: self.open_braces.pop().unwrap(),
                    newline_loc: self.location(),
                })));
            }

            if !self.open_delimiters.is_empty() {
                let (expected, from_loc) = self.open_delimiters.pop().unwrap();
                return Some(Err(Box::new(LexicalError::NewlineInEmphDelimiter {
                    delimiter_start_loc: from_loc,
                    newline_loc: self.location(),
                    expected,
                })));
            }

            self.enqueue(self.span(Tok::Newline));
            let enqueue_par_break = self.try_consume(&PAR_BREAKS).is_some();

            {
                let target = if let Some(indent) = self.try_consume(&WHITESPACE) {
                    indent_level(indent)
                } else {
                    0
                };
                self.enqueue_indentation_level(target);
            }

            if enqueue_par_break {
                self.enqueue(self.span(Tok::ParBreak));
            }

            let ret = self.dequeue().unwrap();
            self.last_tok = Some(ret.1.clone());
            return Some(Ok(ret));
        }
        self.start_of_line = false;

        if self.can_start_attrs() && self.try_consume(&OPEN_ATTRS).is_some() {
            self.parsing_attrs = true;
            return Some(Ok(self.span(Tok::LBracket)));
        }

        match_token! {
            ! => panic!("internal error: unexpected EOF"),

            COMMENT      => |s: &'input str| Ok(Tok::Comment(&s[2..])),
            DOUBLE_COLON => |_| Ok(Tok::DoubleColon),
            COLON        => |_| Ok(Tok::Colon),

            BRACE_LEFT => |_| {
                self.open_braces.push(self.location());
                Ok(Tok::LBrace)
            },
            BRACE_RIGHT => |_| {
                if !self.open_braces.is_empty() {
                    self.open_braces.pop();
                }
                Ok(Tok::RBrace)
            },
            NESTED_COMMENT_OPEN => |_| {
                self.multi_line_comment_starts.push(self.location());
                Ok(Tok::NestedCommentOpen)
            },
            NESTED_COMMENT_CLOSE => |_| {
                Err(Box::new(LexicalError::UnmatchedCommentClose { loc: self.location() }))
            },

            COMMAND    => |s:&'input str| {
                let pluses = s.chars().rev().take_while(|c| *c == '+').count();
                Ok(Tok::Command(&s[1..s.len()-pluses], pluses))
            },
            DASH       => |s:&'input str| Ok(Tok::Dash(s)),
            GLUE       => |s:&'input str| {
                if s.len() == 2 {
                    self.opening_delimiters = true;
                }
                Ok(Tok::Glue(s))
            },
            UNDERSCORES    => |s:&'input str| self.emph(s),
            ASTERISKS      => |s:&'input str| self.emph(s),
            EQUALS         => |s:&'input str| self.emph(s),
            BACKTICKS      => |s:&'input str| self.emph(s),
            VERBATIM   => |s:&'input str| {
                self.opening_delimiters = false;
                Ok(Tok::Verbatim(&s[1..s.len()-1]))
            },
            WORD       => |s:&'input str| {
                self.opening_delimiters = false;
                Ok(Tok::Word(s))
            },
            WHITESPACE => |s:&'input str| {
                self.opening_delimiters = true;
                Ok(Tok::Whitespace(s))
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tok<'input> {
    Indent,
    Dedent,
    Colon,
    DoubleColon,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    NamedAttr(&'input str),
    UnnamedAttr(&'input str),
    AttrComma,
    Command(&'input str, usize),
    ItalicOpen(&'input str),
    BoldOpen(&'input str),
    MonospaceOpen(&'input str),
    SmallcapsOpen(&'input str),
    AlternateFaceOpen(&'input str),
    ItalicClose,
    BoldClose,
    MonospaceClose,
    SmallcapsClose,
    AlternateFaceClose,
    ParBreak,
    Word(&'input str),
    Whitespace(&'input str),
    Dash(&'input str),
    Glue(&'input str),
    Verbatim(&'input str),
    NestedCommentOpen,
    NestedCommentClose,
    Comment(&'input str),
    Newline,
}

impl Display for Tok<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tok::Indent => "indent",
            Tok::Dedent => "dedent",
            Tok::Colon => ":",
            Tok::DoubleColon => "::",
            Tok::LBrace => "{",
            Tok::RBrace => "}",
            Tok::LBracket => "[",
            Tok::RBracket => "]",
            Tok::NamedAttr(_) => "named-attr",
            Tok::UnnamedAttr(_) => "unnamed-attr",
            Tok::AttrComma => "comma",
            Tok::Command(_, _) => "command",
            Tok::ItalicOpen(_) => "italic-open",
            Tok::ItalicClose => "italic-close",
            Tok::BoldOpen(_) => "bold-open",
            Tok::BoldClose => "bold-close",
            Tok::MonospaceOpen(_) => "monospace-open",
            Tok::MonospaceClose => "monospace-close",
            Tok::SmallcapsOpen(_) => "smallcaps-open",
            Tok::SmallcapsClose => "smallcaps-close",
            Tok::AlternateFaceOpen(_) => "alternate-face-open",
            Tok::AlternateFaceClose => "alternate-face-close",
            Tok::ParBreak => "par-break",
            Tok::Word(_) => "word",
            Tok::Whitespace(_) => "whitespace",
            Tok::Dash(_) => "dash",
            Tok::Glue(_) => "glue",
            Tok::Verbatim(_) => "verbatim",
            Tok::NestedCommentOpen => "/*",
            Tok::NestedCommentClose => "*/",
            Tok::Newline => "newline",
            Tok::Comment(_) => "comment",
        }
        .fmt(f)
    }
}

/// Compute the level of indentation for the given string.
fn indent_level(s: &str) -> u32 {
    let mut tabs = 0;
    let mut spaces = 0;

    for chr in s.chars() {
        match chr {
            ' ' => spaces += 1,
            '\t' => tabs += 1,
            _ => break,
        }
    }

    tabs + (spaces as f32 / 4_f32).ceil() as u32
}

pub type SpannedTok<'input> = (Point<'input>, Tok<'input>, Point<'input>);

#[derive(Debug)]
pub enum LexicalError<'input> {
    UnexpectedChar {
        loc: Location<'input>,
        found: char,
    },
    UnexpectedEOF {
        point: Point<'input>,
        expected: Vec<String>,
    },
    UnmatchedCommentOpen {
        unclosed: Vec<Location<'input>>,
    },
    UnmatchedCommentClose {
        loc: Location<'input>,
    },
    NewlineInArg {
        arg_start_loc: Location<'input>,
        newline_loc: Location<'input>,
    },
    NewlineInEmphDelimiter {
        delimiter_start_loc: Location<'input>,
        newline_loc: Location<'input>,
        expected: &'input str,
    },
    DelimiterMismatch {
        loc: Location<'input>,
        to_close_loc: Location<'input>,
        expected: &'input str,
    },
}

impl<'input> Message<'input> for LexicalError<'input> {
    fn log(self) -> Log<'input> {
        match self {
            Self::UnexpectedChar { found, loc } => UnexpectedChar::new(loc, found).log(),
            Self::UnexpectedEOF { point, expected } => UnexpectedEOF::new(point, expected).log(),
            Self::UnmatchedCommentOpen { unclosed } => UnclosedComments::new(unclosed).log(),
            Self::UnmatchedCommentClose { loc } => ExtraCommentClose::new(loc).log(),
            Self::NewlineInArg {
                arg_start_loc,
                newline_loc,
            } => NewlineInInlineArg::new(arg_start_loc, newline_loc).log(),
            Self::NewlineInEmphDelimiter {
                delimiter_start_loc,
                newline_loc,
                expected,
            } => NewlineInEmphDelimiter::new(delimiter_start_loc, newline_loc, expected).log(),
            Self::DelimiterMismatch {
                loc,
                to_close_loc,
                expected,
            } => DelimiterMismatch::new(loc, to_close_loc, expected).log(),
        }
    }
}

impl Error for LexicalError<'_> {}

impl Display for LexicalError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedChar { found, loc } => {
                write!(f, "unexpected character '{:?}' found at {}", found, loc)
            }
            Self::UnexpectedEOF { point, .. } => write!(f, "unexpected EOF found at {}", point),
            Self::UnmatchedCommentOpen { unclosed } => write!(
                f,
                "unclosed comment found at {:?}",
                unclosed.iter().map(|u| u.to_string()).collect::<Vec<_>>()
            ),
            Self::UnmatchedCommentClose { loc, .. } => {
                write!(f, "no comment to close found at {}", loc)
            }
            Self::NewlineInArg { arg_start_loc, .. } => {
                write!(f, "newline in braced args found at {}", arg_start_loc)
            }
            Self::NewlineInEmphDelimiter {
                newline_loc,
                expected,
                ..
            } => {
                write!(
                    f,
                    "newline in {:?} emphasis found at {}",
                    expected, newline_loc
                )
            }
            Self::DelimiterMismatch {
                loc,
                to_close_loc,
                expected,
            } => {
                write!(
                    f,
                    "delimiter mismatch for {} found at {} (failed to match at {})",
                    expected, loc, to_close_loc
                )
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_indent_str(expected: u32, s: &str) {
        assert_eq!(expected, indent_level(s));
        assert_eq!(expected, indent_level(&format!("{}foo", s)));
        assert_eq!(expected, indent_level(&format!("{}foo{}", s, s)));
    }

    #[test]
    fn indent_level_counting() {
        test_indent_str(0, "");
        test_indent_str(1, " ");
        test_indent_str(1, "\t");
        test_indent_str(1, "    ");
        test_indent_str(2, "\t ");
        test_indent_str(2, " \t ");
        test_indent_str(2, "\t\t");
        test_indent_str(2, "        ");
        test_indent_str(3, "    \t    ");
    }
}
