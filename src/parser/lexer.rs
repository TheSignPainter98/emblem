use crate::parser::location::Location;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
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
    let WORD               = r"[^ \t\r\n}]+";
    let WHITESPACE         = r"[ \t]+";
    let PAR_BREAKS         = r"([ \t]*(\n|\r\n|\r))+";
    let LN                 = r"(\n|\r\n|\r)";
    let COLON              = r":";
    let DOUBLE_COLON       = r"::";
    let INITIAL_INDENT     = r"[ \t]*";
    let COMMAND            = r"\.[^ \t{}\r\n]+";
    let BRACE_LEFT         = r"\{";
    let BRACE_RIGHT        = r"\}";
    let COMMENT            = r"//[^\r\n]*";

    let NESTED_COMMENT_OPEN  = r"/\*";
    let NESTED_COMMENT_CLOSE = r"\*/";
    let NESTED_COMMENT_PART  = r"([^*/\n\r]|\*[^/\n\r]|/[^*\n\r])+";
}

pub struct Lexer<'input> {
    input: &'input str,
    failed: bool,
    insert_par_break: bool,
    current_indent: u32,
    target_indent: u32,
    curr_loc: Location<'input>,
    prev_loc: Location<'input>,
    comment_depth: u32,
}

impl<'input> Lexer<'input> {
    pub fn new(file: &'input str, input: &'input str) -> Self {
        Self {
            input,
            failed: false,
            insert_par_break: false,
            current_indent: 0,
            target_indent: 0,
            curr_loc: Location::new(file, input),
            prev_loc: Location::new(file, input),
            comment_depth: 0,
        }
    }

    fn try_consume(&mut self, re: &Regex) -> Option<&'input str> {
        if let Some(mat) = re.find(self.input) {
            self.input = &self.input[mat.end()..];

            self.prev_loc = self.curr_loc;
            self.curr_loc = self.curr_loc.shift(mat.as_str());

            Some(mat.as_str())
        } else {
            None
        }
    }

    fn span(&self, tok: Tok<'input>) -> SpannedTok<'input> {
        (self.prev_loc.clone(), tok, self.curr_loc.clone())
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<SpannedTok<'input>, LexicalError<'input>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }

        macro_rules! match_token {
            ( $($re:expr => $to_tok:expr),* $(,)? ) => {
                if false { None }
                $(else if let Some(mat) = self.try_consume(&$re) {
                    Some($to_tok(mat).map(|t| self.span(t)))
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
                NESTED_COMMENT_PART  => |s: &'input str| Ok(Tok::Comment(s.trim())),
                LN                   => |_| Ok(Tok::Newline),
                NESTED_COMMENT_OPEN  => |_| {
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
            self.target_indent = 0;
        } else if self.try_consume(&LN).is_some() {
            self.insert_par_break = self.try_consume(&PAR_BREAKS).is_some();

            let mat = self.try_consume(&INITIAL_INDENT).unwrap();
            self.target_indent = indent_level(mat);

            if self.insert_par_break && self.current_indent < self.target_indent {
                self.insert_par_break = false;
                return Some(Ok(self.span(Tok::ParBreak)));
            }
        } else if self.curr_loc.index == 0 {
            if let Some(mat) = self.try_consume(&INITIAL_INDENT) {
                self.target_indent = indent_level(mat);
            }
        }

        if self.current_indent != self.target_indent {
            if self.current_indent < self.target_indent {
                self.current_indent += 1;
                return Some(Ok(self.span(Tok::Indent)));
            } else {
                self.current_indent -= 1;
                return Some(Ok(self.span(Tok::Dedent)));
            }
        }

        if self.input.is_empty() {
            return None;
        }

        if self.insert_par_break {
            self.insert_par_break = false;
            return Some(Ok(self.span(Tok::ParBreak)));
        }

        match_token! {
            COMMENT              => |s: &'input str| Ok(Tok::Comment(s[2..].trim())),
            DOUBLE_COLON         => |_| Ok(Tok::DoubleColon),
            COLON                => |_| Ok(Tok::Colon),
            BRACE_LEFT           => |_| Ok(Tok::LBrace),
            BRACE_RIGHT          => |_| Ok(Tok::RBrace),
            NESTED_COMMENT_OPEN  => |_| {
                self.comment_depth += 1;
                Ok(Tok::NestedCommentOpen)
            },
            NESTED_COMMENT_CLOSE => |_| {
                if self.comment_depth == 0 {
                    self.failed = true;
                    Err(LexicalError{
                        reason: LexicalErrorReason::UnmatchedCommentClose,
                        loc: self.curr_loc.clone(),
                    })
                } else {
                    Ok(Tok::NestedCommentClose)
                }
            },
            COMMAND    => |s:&'input str| Ok(Tok::Command(&s[1..])),
            WORD       => |s:&'input str| Ok(Tok::Word(s)),
            WHITESPACE => |s:&'input str| Ok(Tok::Whitespace(s)),
        }
    }
}

#[derive(Clone, Debug)]
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
    NestedCommentOpen,
    NestedCommentClose,
    Comment(&'input str),
    Newline,
}

impl ToString for Tok<'_> {
    fn to_string(&self) -> String {
        match self {
            Tok::Indent => "indent",
            Tok::Dedent => "dedent",
            Tok::Colon => ":",
            Tok::DoubleColon => "::",
            Tok::LBrace => "{",
            Tok::RBrace => "}",
            Tok::Command(_) => "command",
            Tok::ParBreak => "paragraph break",
            Tok::Word(_) => "word",
            Tok::Whitespace(_) => "whitespace",
            Tok::NestedCommentOpen => "/*",
            Tok::NestedCommentClose => "*/",
            Tok::Newline => "newline",
            Tok::Comment(_) => "comment",
        }
        .to_owned()
    }
}

fn indent_level(s: &str) -> u32 {
    let mut tabs = 0;
    let mut spaces = 0;

    for chr in s.chars() {
        match chr {
            ' ' => spaces += 1,
            '\t' => tabs += 1,
            _ => {}
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
}

impl LexicalErrorReason {
    fn description(&self) -> &str {
        match self {
            LexicalErrorReason::UnexpectedChar(_) => "unexpected character",
            LexicalErrorReason::UnmatchedCommentClose => "no comment to close",
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

    #[test]
    fn indent_level_counting() {
        assert_eq!(0, indent_level(""));
        assert_eq!(1, indent_level(" "));
        assert_eq!(1, indent_level("\t"));
        assert_eq!(1, indent_level("    "));
        assert_eq!(2, indent_level("\t "));
        assert_eq!(2, indent_level(" \t "));
        assert_eq!(2, indent_level("\t\t"));
        assert_eq!(2, indent_level("        "));
        assert_eq!(3, indent_level("    \t    "));
    }
}
