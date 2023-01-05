use crate::parser::location::Location;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::VecDeque,
    error::Error,
    fmt::{self, Display},
};

macro_rules! token_patterns {
    ( $(let $name:ident = $pattern:literal);* $(;)? ) => {
        lazy_static! {
            $(static ref $name: Regex = Regex::new(concat!("^", $pattern)).unwrap();)*
        }
    }
}

token_patterns! {
    let WORD               = r"([^ /\t\r\n}]|/[^ /\t\r\n])+";
    let WHITESPACE         = r"[ \t]+";
    let PAR_BREAKS         = r"([ \t]*(\n|\r\n|\r))+";
    let LN                 = r"(\n|\r\n|\r)";
    let COLON              = r":";
    let DOUBLE_COLON       = r"::";
    let INITIAL_INDENT     = r"[ \t]*";
    let COMMAND            = r"\.[^ \t{}\r\n]+";
    let ESCAPED_COMMAND    = r"\\\.[^ \t{}\r\n]+";
    let BRACE_LEFT         = r"\{";
    let BRACE_RIGHT        = r"\}";
    let COMMENT            = r"//[^\r\n]*";

    let NESTED_COMMENT_OPEN  = r"/\*";
    let NESTED_COMMENT_CLOSE = r"\*/";
    let NESTED_COMMENT_PART  = r"([^*/\n\r]|\*[^/\n\r]|/[^*\n\r])+";
}

pub struct Lexer<'input> {
    input: &'input str,
    done: bool,
    failed: bool,
    start_of_line: bool,
    current_indent: u32,
    curr_loc: Location<'input>,
    prev_loc: Location<'input>,
    open_braces: u32,
    next_toks: VecDeque<SpannedTok<'input>>,
    comment_depth: u32,
    last_tok: Option<Tok<'input>>,
}

impl<'input> Lexer<'input> {
    pub fn new(file: &'input str, input: &'input str) -> Self {
        Self {
            input,
            done: false,
            failed: false,
            start_of_line: true,
            current_indent: 0,
            curr_loc: Location::new(file, input),
            prev_loc: Location::new(file, input),
            open_braces: 0,
            next_toks: VecDeque::new(),
            comment_depth: 0,
            last_tok: None,
        }
    }

    fn try_consume(&mut self, re: &Regex) -> Option<&'input str> {
        if let Some(mat) = re.find(self.input) {
            self.input = &self.input[mat.end()..];

            let curr_loc = self.curr_loc.clone();
            self.prev_loc = curr_loc.clone();
            self.curr_loc = curr_loc.shift(mat.as_str());

            Some(mat.as_str())
        } else {
            None
        }
    }

    fn span(&self, tok: Tok<'input>) -> SpannedTok<'input> {
        (self.prev_loc.clone(), tok, self.curr_loc.clone())
    }

    fn enqueue_indentation_delta(&mut self, target: u32) {
        let difference = self.current_indent.abs_diff(target);
        let tok = if self.current_indent > target {
            Tok::Indent
        } else {
            Tok::Dedent
        };

        for _ in 1..difference {
            self.enqueue(self.span(tok.clone()))
        }
    }

    fn dequeue(&mut self) -> Option<SpannedTok<'input>> {
        self.next_toks.pop_front()
    }

    fn enqueue(&mut self, t: SpannedTok<'input>) {
        self.next_toks.push_back(t)
    }

    fn push(&mut self, t: SpannedTok<'input>) {
        self.next_toks.push_front(t)
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<SpannedTok<'input>, LexicalError<'input>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }

        if let Some(t) = self.dequeue() {
            return Some(Ok(t));
        }

        if self.done {
            return None;
        }

        macro_rules! match_token {
            ( $($re:expr => $to_tok:expr),* $(,)? ) => {
                if false { None }
                $(else if let Some(mat) = self.try_consume(&$re) {
                    let ret = $to_tok(mat).map(|t| self.span(t));
                    self.last_tok = ret.as_ref().ok().map(|s| s.1.clone());
                    Some(ret)
                })*
                else {
                    self.failed = true;
                    Some(Err(LexicalError {
                        reason: LexicalErrorReason::UnexpectedChar(self.input.chars().next().unwrap_or('\0')),
                        loc: self.curr_loc.clone(),
                    }))
                }
            };
        }

        if self.comment_depth > 0 {
            return match_token![
                NESTED_COMMENT_PART => |s: &'input str| {
                    if self.start_of_line {
                        self.current_indent = indent_level(s);
                    }
                    Ok(Tok::Comment(s))
                },
                LN => |_| {
                    self.start_of_line = true;
                    Ok(Tok::Newline)
                },
                NESTED_COMMENT_OPEN => |_| {
                    self.comment_depth += 1;
                    Ok(Tok::NestedCommentOpen)
                },
                NESTED_COMMENT_CLOSE => |_| {
                    self.comment_depth -= 1;
                    Ok(Tok::NestedCommentClose)
                },
            ];
        }

        if self.input.is_empty() {
            if self.last_tok != Some(Tok::Newline) {
                self.enqueue(self.span(Tok::Newline));
            }
            self.enqueue_indentation_delta(0);
            self.done = true;
            return self.dequeue().map(|t| Ok(t));
        }

        if self.try_consume(&LN).is_some() {
            self.start_of_line = true;

            if self.open_braces > 0 {
                self.failed = true;
                return Some(Err(LexicalError {
                    reason: LexicalErrorReason::NewlineInArg,
                    loc: self.curr_loc.clone(),
                }));
            }

            self.enqueue(self.span(Tok::Newline));

            if self.try_consume(&PAR_BREAKS).is_some() {
                self.enqueue(self.span(Tok::ParBreak));
            }

            let ret = self.dequeue().unwrap();
            self.last_tok = Some(ret.1.clone());
            return Some(Ok(ret))
        }
        self.start_of_line = false;

        match_token! {
            COMMENT              => |s: &'input str| Ok(Tok::Comment(&s[2..])),
            DOUBLE_COLON         => |_| Ok(Tok::DoubleColon),
            COLON                => |_| Ok(Tok::Colon),
            BRACE_LEFT           => |_| {
                self.open_braces += 1;
                Ok(Tok::LBrace)
            },
            BRACE_RIGHT          => |_| {
                if self.open_braces > 0 {
                    self.open_braces -= 1;
                }
                Ok(Tok::RBrace)
            },
            NESTED_COMMENT_OPEN  => |_| {
                self.comment_depth = 1;
                Ok(Tok::NestedCommentOpen)
            },
            NESTED_COMMENT_CLOSE => |_| {
                self.failed = true;
                Err(LexicalError{
                    reason: LexicalErrorReason::UnmatchedCommentClose,
                    loc: self.curr_loc.clone(),
                })
            },
            COMMAND         => |s:&'input str| Ok(Tok::Command(&s[1..])),
            ESCAPED_COMMAND => |s:&'input str| Ok(Tok::Word(&s[1..])),
            WORD            => |s:&'input str| Ok(Tok::Word(s)),
            WHITESPACE      => |s:&'input str| Ok(Tok::Whitespace(s)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tok<'input> {
    Indent,
    Dedent,
    Colon,
    DoubleColon,
    LBrace,
    RBrace,
    Command(&'input str),
    ParBreak,
    Word(&'input str),
    Whitespace(&'input str),
    Dash(&'input str),
    Glue(&'input str),
    NestedCommentOpen,
    NestedCommentClose,
    Comment(&'input str),
    Newline,
}

// impl ToString for Tok<'_> {
//     fn to_string(&self) -> String {
//         match self {
//             Tok::Indent => "indent",
//             Tok::Dedent => "dedent",
//             Tok::Colon => ":",
//             Tok::DoubleColon => "::",
//             Tok::LBrace => "{",
//             Tok::RBrace => "}",
//             Tok::Command(_) => "command",
//             Tok::ParBreak => "paragraph break",
//             Tok::Word(_) => "word",
//             Tok::Whitespace(_) => "whitespace",
//             Tok::NestedCommentOpen => "/*",
//             Tok::NestedCommentClose => "*/",
//             Tok::Newline => "newline",
//             Tok::Comment(_) => "comment",
//         }
//         .to_owned()
//     }
// }

impl Display for Tok<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tok::Indent => write!(f, "(indent)"),
            Tok::Dedent => write!(f, "(dedent)"),
            Tok::Colon => write!(f, "(:)"),
            Tok::DoubleColon => write!(f, "(::)"),
            Tok::LBrace => write!(f, "({{)"),
            Tok::RBrace => write!(f, "(}})"),
            Tok::Command(c) => write!(f, "(.{})", c),
            Tok::ParBreak => write!(f, "(paragraph break)"),
            Tok::Word(w) => write!(f, "({})", w),
            Tok::Whitespace(w) => write!(f, "(whitespace:{})", w),
            Tok::Dash(d) => write!(f, "(dash:{})", d),
            Tok::Glue(g) => write!(f, "(glue:{})", g),
            Tok::NestedCommentOpen => write!(f, "(/*)"),
            Tok::NestedCommentClose => write!(f, "(*/)"),
            Tok::Newline => write!(f, "(newline)"),
            Tok::Comment(c) => write!(f, "(// {})", c),
        }
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

pub type SpannedTok<'input> = (Location<'input>, Tok<'input>, Location<'input>);

#[derive(Debug)]
pub struct LexicalError<'input> {
    reason: LexicalErrorReason,
    loc: Location<'input>,
}

impl<'input> LexicalError<'input> {
    pub fn location(&self) -> &Location {
        &self.loc
    }
}

impl Display for LexicalError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.reason.fmt(f)
    }
}

impl Error for LexicalError<'_> {
    fn description(&self) -> &str {
        self.reason.description()
    }
}

#[derive(Debug)]
enum LexicalErrorReason {
    UnexpectedChar(char),
    UnmatchedCommentClose,
    NewlineInArg,
}

impl LexicalErrorReason {
    fn description(&self) -> &str {
        match self {
            LexicalErrorReason::UnexpectedChar(_) => "unexpected character",
            LexicalErrorReason::UnmatchedCommentClose => "no comment to close",
            LexicalErrorReason::NewlineInArg => "newline in brace args",
        }
    }
}

impl Display for LexicalErrorReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
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
