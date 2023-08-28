use crate::context::file_content::FileSlice;
use crate::log::messages::{
    DelimiterMismatch, EmptyQualifier, ExtraCommentClose, HeadingTooDeep, NewlineInAttrs,
    NewlineInEmphDelimiter, NewlineInInlineArg, TooManyQualifiers, UnclosedComments,
    UnexpectedChar, UnexpectedEOF, UnexpectedHeading,
};
use crate::log::Log;
use crate::parser::Location;
use crate::{log::messages::Message, parser::point::Point};
use crate::{FileContent, FileContentSlice, FileName};
use lazy_static::lazy_static;
use regex::Regex;
use uniquote::Quote;

use std::{
    collections::VecDeque,
    error::Error,
    fmt::{self, Display},
};

pub struct Lexer {
    input: FileContentSlice,
    done: bool,
    failed: bool,
    start_of_line: bool,
    current_indent: u32,
    curr_point: Point,
    prev_point: Point,
    open_braces: Vec<Location>,
    next_toks: VecDeque<SpannedTok>,
    multi_line_comment_starts: Vec<Location>,
    last_tok: Option<Tok>,
    attr_open: Option<Location>,
    opening_delimiters: bool,
    open_delimiters: Vec<(FileContentSlice, Location)>,
}

impl Lexer {
    pub fn new(file: FileName, input: FileContent) -> Self {
        Self {
            input: input.clone().into(),
            done: false,
            failed: false,
            start_of_line: true,
            current_indent: 0,
            curr_point: Point::at_start_of(file.clone(), input.clone()),
            prev_point: Point::at_start_of(file, input),
            open_braces: Vec::new(),
            next_toks: VecDeque::new(),
            multi_line_comment_starts: Vec::new(),
            last_tok: None,
            attr_open: None,
            opening_delimiters: true,
            open_delimiters: Vec::new(),
        }
    }

    fn try_consume(&mut self, re: &Regex) -> Option<FileContentSlice> {
        if let Some(mat) = re.find(self.input.to_str()) {
            let curr_point = self.curr_point.clone();
            self.prev_point = curr_point.clone();
            self.curr_point = curr_point.shift(mat.as_str());

            let ret = self.input.slice(mat.range());
            self.input = self.input.slice(mat.end()..);
            Some(ret)
        } else {
            None
        }
    }

    fn span(&self, tok: Tok) -> SpannedTok {
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

    fn dequeue(&mut self) -> Option<SpannedTok> {
        self.next_toks.pop_front()
    }

    fn enqueue(&mut self, t: SpannedTok) {
        self.next_toks.push_back(t)
    }

    fn can_start_attrs(&self) -> bool {
        matches!(self.last_tok, Some(Tok::Command { .. }))
    }

    fn location(&self) -> Location {
        Location::new(&self.prev_point, &self.curr_point)
    }

    fn emph(&mut self, raw: FileContentSlice) -> Result<Tok, Box<LexicalError>> {
        if self.opening_delimiters {
            self.open_delimiters.push((raw.clone(), self.location()));

            return match raw.to_str() {
                "_" | "*" => Ok(Tok::ItalicOpen(raw)),
                "__" | "**" => Ok(Tok::BoldOpen(raw)),
                "=" => Ok(Tok::SmallcapsOpen),
                "==" => Ok(Tok::AlternateFaceOpen),
                "`" => Ok(Tok::MonospaceOpen),
                _ => panic!("internal error: unknown emphasis string {:?}", raw),
            };
        }

        if !self.open_delimiters.is_empty() {
            let (to_close, to_close_loc) = self.open_delimiters.pop().unwrap();
            if to_close != raw {
                self.failed = true;
                return Err(Box::new(LexicalError::DelimiterMismatch {
                    loc: self.location(),
                    to_close_loc,
                    expected: to_close,
                }));
            }
        }

        match raw.to_str() {
            "_" | "*" => Ok(Tok::ItalicClose),
            "__" | "**" => Ok(Tok::BoldClose),
            "=" => Ok(Tok::SmallcapsClose),
            "==" => Ok(Tok::AlternateFaceClose),
            "`" => Ok(Tok::MonospaceClose),
            _ => panic!("internal error: unknown emphasis string {:?}", raw),
        }
    }
}

impl Iterator for Lexer {
    type Item = Result<SpannedTok, Box<LexicalError>>;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! token_patterns {
            ( $(let $name:ident = $pattern:literal);* $(;)? ) => {
                lazy_static! {
                    $(static ref $name: Regex = Regex::new(concat!("^", $pattern)).unwrap();)*
                }
            }
        }

        token_patterns! {
            let SHEBANG = r"#![^\r\n]*";

            let WORD           = r"([^ /\t\r\n}_*`=~-]|/[^ /\t\r\n}_*`=~-])+";
            let WHITESPACE     = r"[ \t]+";
            let PAR_BREAKS     = r"([ \t]*(\n|\r\n|\r))+";
            let LN             = r"(\n|\r\n|\r)";
            let COLON          = r":[ \t]*";
            let DOUBLE_COLON   = r"::";
            let INITIAL_INDENT = r"[ \t]*";
            let VERBATIM       = r"![^!\r\n]+!";
            let BRACE_LEFT     = r"\{";
            let BRACE_RIGHT    = r"\}";
            let COMMENT        = r"//[^\r\n]*";
            let DASH           = r"-{1,3}";
            let GLUE           = r" *~~? *";
            let UNDERSCORES    = r"_{1,2}";
            let ASTERISKS      = r"\*{1,2}";
            let EQUALS         = r"={1,2}";
            let BACKTICKS      = r"`";
            let HEADING        = r"#+\+*";
            let MARK           = r#"@[^ \t\r\n#+.,?!'"(){}\[\]]+"#;
            let REFERENCE      = r#"#[^ \t\r\n#+.,?!'"(){}\[\]]+"#;

            let QUALIFIED_COMMAND = r"(\.+[^ \t{}\[\]\r\n:+.]*){2,}[^ \t{}\[\]\r\n:+.]\+*";
            let COMMAND           = r"\.[^ \t{}\[\]\r\n:+.]+\+*";

            let OPEN_ATTRS   = r"\[";
            let CLOSE_ATTRS  = r"]";
            let COMMA        = r",";
            let UNNAMED_ATTR = r"[ \t]*([^,= \r\n\t\[\]]|\\[,=\[\]])+[ \t]*";
            let NAMED_ATTR   = r"[ \t]*([^,= \r\n\t\[\]]|\\[,=\[\]])+[ \t]*=[ \t]*([^,\[\]\r\n]|\\[,\[\]])*[ \t]*";

            let NESTED_COMMENT_OPEN  = r"/\*";
            let NESTED_COMMENT_CLOSE = r"\*/";
            let NESTED_COMMENT_PART  = r"([^*/\r\n]|\*[^/\r\n]|/[^*\r\n])+";
        }

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
            ( ! => $on_eof:expr, $($re:ident => $to_tok:expr),* $(,)? ) => {
                if self.input.is_empty() {
                    #[allow(unreachable_code)]
                    Some($on_eof)
                }
                $(else if let Some(mat) = self.try_consume(&$re) {
                    let mat: FileContentSlice = mat; // assert $to_tok input type
                    let ret = $to_tok(mat).map(|t: Tok| self.span(t));
                    self.last_tok = ret.as_ref().ok().map(|s| s.1.clone());
                    Some(ret)
                })*
                else {
                    self.failed = true;
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
                    self.failed = true;
                    Err(Box::new(LexicalError::UnmatchedCommentOpen {
                        unclosed: self.multi_line_comment_starts.clone(),
                    }))
                },

                NESTED_COMMENT_PART => |s: FileContentSlice| Ok(Tok::Comment(s)) ,
                LN                  => |_| Ok(Tok::Newline{ at_eof: false }) ,
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

        if let Some(attr_start_loc) = &self.attr_open {
            let attr_start_loc = attr_start_loc.clone();
            return match_token! {
                ! => {
                    self.failed = true;
                    Err(Box::new(LexicalError::UnexpectedEOF {
                        point: self.curr_point.clone(),
                        expected: vec![],
                    }))
                },

                NAMED_ATTR   => |s: FileContentSlice| Ok(Tok::NamedAttr(s)),
                UNNAMED_ATTR => |s: FileContentSlice| Ok(Tok::UnnamedAttr(s)),
                COMMA        => |_| Ok(Tok::AttrComma),
                OPEN_ATTRS   => |s: FileContentSlice| {
                    Err(Box::new(LexicalError::UnexpectedChar {
                        found: s.chars().next().unwrap(),
                        loc: self.location(),
                    }))
                },
                CLOSE_ATTRS => |_| {
                    self.attr_open = None;
                    Ok(Tok::RBracket)
                },
                LN => |_| {
                    self.attr_open = None;
                    Err(Box::new(LexicalError::NewlineInAttrs {
                        attr_start_loc: attr_start_loc.clone(),
                        newline_loc: self.location(),
                    }))
                },
            };
        }

        if self.input.is_empty() {
            if !self.open_braces.is_empty() {
                self.failed = true;
                return Some(Err(Box::new(LexicalError::UnexpectedEOF {
                    point: self.curr_point.clone(),
                    expected: vec!["\"}\"".into()],
                })));
            }

            if !matches!(self.last_tok, Some(Tok::Newline { .. })) {
                self.enqueue(self.span(Tok::Newline { at_eof: true }));
            }
            self.enqueue_indentation_level(0);
            self.done = true;
            return self.dequeue().map(Ok);
        }

        if self.try_consume(&LN).is_some() {
            self.start_of_line = true;
            self.opening_delimiters = true;

            if !self.open_braces.is_empty() {
                self.failed = true;
                return Some(Err(Box::new(LexicalError::NewlineInArg {
                    arg_start_loc: self.open_braces.pop().unwrap(),
                    newline_loc: self.location(),
                })));
            }

            if !self.open_delimiters.is_empty() {
                self.failed = true;
                let (expected, from_loc) = self.open_delimiters.pop().unwrap();
                return Some(Err(Box::new(LexicalError::NewlineInEmphDelimiter {
                    delimiter_start_loc: from_loc,
                    newline_loc: self.location(),
                    expected,
                })));
            }

            self.enqueue(self.span(Tok::Newline { at_eof: false }));
            let enqueue_par_break = self.try_consume(&PAR_BREAKS).is_some();

            {
                let target = if let Some(indent) = self.try_consume(&WHITESPACE) {
                    indent_level(indent.to_str())
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

        // Avoid clash with heading '#'
        if let Some(reference) = &self.try_consume(&REFERENCE) {
            return Some(Ok(self.span(Tok::Reference(reference.slice(1..)))));
        }

        if self.start_of_line {
            if self.curr_point.index() == 0 {
                if let Some(shebang) = &self.try_consume(&SHEBANG) {
                    return Some(Ok(self.span(Tok::Shebang(shebang.slice(2..)))));
                }
            }

            if let Some(heading) = &self.try_consume(&HEADING) {
                self.start_of_line = false;
                let heading = heading.trimmed_right();

                let level = heading.find('+').unwrap_or(heading.len());
                if level > 6 {
                    return Some(Err(Box::new(LexicalError::HeadingTooDeep {
                        loc: self.location(),
                        level,
                    })));
                }

                let pluses = heading.len() - level;
                return Some(Ok(self.span(Tok::Heading { level, pluses })));
            }
        }

        let line_started_before_match = self.start_of_line;
        self.start_of_line = false;

        if self.can_start_attrs() && self.try_consume(&OPEN_ATTRS).is_some() {
            self.attr_open = Some(self.location());
            return Some(Ok(self.span(Tok::LBracket)));
        }

        match_token! {
            ! => panic!("internal error: unexpected EOF"),

            COMMENT      => |s: FileContentSlice| Ok(Tok::Comment(s.slice(2..))),
            DOUBLE_COLON => |_| Ok(Tok::DoubleColon),
            COLON        => |_| Ok(Tok::Colon),

            BRACE_LEFT => |_| {
                self.open_braces.push(self.location());
                Ok(Tok::LBrace)
            },
            BRACE_RIGHT => |_| {
                self.open_braces.pop();
                Ok(Tok::RBrace)
            },
            NESTED_COMMENT_OPEN => |_| {
                self.multi_line_comment_starts.push(self.location());
                Ok(Tok::NestedCommentOpen)
            },
            NESTED_COMMENT_CLOSE => |_| {
                self.failed = true;
                Err(Box::new(LexicalError::UnmatchedCommentClose { loc: self.location() }))
            },

            QUALIFIED_COMMAND => |s: FileContentSlice| {
                let pluses = s.chars().rev().take_while(|c| *c == '+').count();
                let dot_idx = 1 + s.to_str()[1..].find('.').unwrap();

                let name = s.slice(1+dot_idx..s.len()-pluses);
                if name.contains('.') {
                    let loc = self.location();
                    let extra_dots_start = loc.start().shift(&s[..1+dot_idx]);
                    let dot_locs = name.char_indices()
                        .filter_map(|(i, c)| if c == '.' { Some(i) } else { None })
                        .map(|i| {
                            let dot_pos = extra_dots_start.clone().shift(&s[..i]);
                            Location::new(&dot_pos, &dot_pos.clone().shift("."))
                        })
                        .collect();
                    return Err(Box::new(LexicalError::TooManyQualifiers { loc, dot_locs }));
                }

                if dot_idx == 1 {
                    let loc = self.location();
                    let start = loc.start();
                    let qualifier_loc = Location::new(&start, &start.clone().shift(".."));
                    return Err(Box::new(LexicalError::EmptyQualifier { loc, qualifier_loc }));
                }

                let qualifier = Some(s.slice(1..dot_idx));
                Ok(Tok::Command{
                    qualifier,
                    name,
                    pluses
                })
            },
            COMMAND => |s: FileContentSlice| {
                let pluses = s.chars().rev().take_while(|c| *c == '+').count();
                let name = s.slice(1..s.len()-pluses);
                Ok(Tok::Command{
                    qualifier: None,
                    name,
                    pluses,
                })
            },
            DASH => |s: FileContentSlice| Ok(Tok::Dash(s)),
            GLUE => |s: FileContentSlice| {
                let newline_after = matches!(self.input.chars().next(), None | Some('\r') | Some('\n'));
                if newline_after
                    || line_started_before_match
                    || s.starts_with(' ')
                    || s.ends_with(' ') {
                    self.opening_delimiters = true;
                    return Ok(Tok::SpiltGlue(s));
                }

                // Captured text is either ~ or ~~
                if s.len() == 2 {
                    self.opening_delimiters = true;
                }
                Ok(Tok::Glue(s))
            },
            UNDERSCORES => |s: FileContentSlice| self.emph(s),
            ASTERISKS   => |s: FileContentSlice| self.emph(s),
            EQUALS      => |s: FileContentSlice| self.emph(s),
            BACKTICKS   => |s: FileContentSlice| self.emph(s),
            HEADING     => |_| Err(Box::new(LexicalError::UnexpectedHeading{ loc: self.location() })),
            MARK        => |s: FileContentSlice| Ok(Tok::Mark(s.slice(1..))),
            VERBATIM    => |s: FileContentSlice| {
                self.opening_delimiters = false;
                Ok(Tok::Verbatim(s.slice(1..s.len()-1)))
            },
            WORD => |s: FileContentSlice| {
                self.opening_delimiters = false;
                Ok(Tok::Word(s))
            },
            WHITESPACE => |s: FileContentSlice| {
                self.opening_delimiters = true;
                Ok(Tok::Whitespace(s))
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tok {
    Shebang(FileContentSlice),
    Indent,
    Dedent,
    Colon,
    DoubleColon,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    NamedAttr(FileContentSlice),
    UnnamedAttr(FileContentSlice),
    AttrComma,
    Command {
        qualifier: Option<FileContentSlice>,
        name: FileContentSlice,
        pluses: usize,
    },
    ItalicOpen(FileContentSlice),
    BoldOpen(FileContentSlice),
    MonospaceOpen,
    SmallcapsOpen,
    AlternateFaceOpen,
    Heading {
        level: usize,
        pluses: usize,
    },
    ItalicClose,
    BoldClose,
    MonospaceClose,
    SmallcapsClose,
    AlternateFaceClose,
    Reference(FileContentSlice),
    Mark(FileContentSlice),
    ParBreak,
    Word(FileContentSlice),
    Whitespace(FileContentSlice),
    Dash(FileContentSlice),
    Glue(FileContentSlice),
    SpiltGlue(FileContentSlice),
    Verbatim(FileContentSlice),
    NestedCommentOpen,
    NestedCommentClose,
    Comment(FileContentSlice),
    Newline {
        at_eof: bool,
    },
}

impl Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tok::Shebang(_) => "shebang",
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
            Tok::Command { .. } => "command",
            Tok::ItalicOpen(_) => "italic-open",
            Tok::ItalicClose => "italic-close",
            Tok::BoldOpen(_) => "bold-open",
            Tok::BoldClose => "bold-close",
            Tok::MonospaceOpen => "monospace-open",
            Tok::MonospaceClose => "monospace-close",
            Tok::SmallcapsOpen => "smallcaps-open",
            Tok::SmallcapsClose => "smallcaps-close",
            Tok::AlternateFaceOpen => "alternate-face-open",
            Tok::AlternateFaceClose => "alternate-face-close",
            Tok::Heading { .. } => "heading",
            Tok::Reference(_) => "reference",
            Tok::Mark(_) => "mark",
            Tok::ParBreak => "par-break",
            Tok::Word(_) => "word",
            Tok::Whitespace(_) => "whitespace",
            Tok::Dash(_) => "dash",
            Tok::Glue(_) => "glue",
            Tok::SpiltGlue(_) => "spilt-glue",
            Tok::Verbatim(_) => "verbatim",
            Tok::NestedCommentOpen => "/*",
            Tok::NestedCommentClose => "*/",
            Tok::Newline { .. } => "newline",
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

pub type SpannedTok = (Point, Tok, Point);

#[derive(Debug)]
pub enum LexicalError {
    UnexpectedChar {
        loc: Location,
        found: char,
    },
    UnexpectedEOF {
        point: Point,
        expected: Vec<String>,
    },
    UnmatchedCommentOpen {
        unclosed: Vec<Location>,
    },
    UnmatchedCommentClose {
        loc: Location,
    },
    NewlineInArg {
        arg_start_loc: Location,
        newline_loc: Location,
    },
    NewlineInAttrs {
        attr_start_loc: Location,
        newline_loc: Location,
    },
    NewlineInEmphDelimiter {
        delimiter_start_loc: Location,
        newline_loc: Location,
        expected: FileContentSlice,
    },
    DelimiterMismatch {
        loc: Location,
        to_close_loc: Location,
        expected: FileContentSlice,
    },
    UnexpectedHeading {
        loc: Location,
    },
    HeadingTooDeep {
        loc: Location,
        level: usize,
    },
    TooManyQualifiers {
        loc: Location,
        dot_locs: Vec<Location>,
    },
    EmptyQualifier {
        loc: Location,
        qualifier_loc: Location,
    },
}

impl Message for LexicalError {
    fn log(self) -> Log {
        match self {
            Self::UnexpectedChar { found, loc } => UnexpectedChar::new(loc, found).log(),
            Self::UnexpectedEOF { point, expected } => UnexpectedEOF::new(point, expected).log(),
            Self::UnmatchedCommentOpen { unclosed } => UnclosedComments::new(unclosed).log(),
            Self::UnmatchedCommentClose { loc } => ExtraCommentClose::new(loc).log(),
            Self::NewlineInArg {
                arg_start_loc,
                newline_loc,
            } => NewlineInInlineArg::new(arg_start_loc, newline_loc).log(),
            Self::NewlineInAttrs {
                attr_start_loc,
                newline_loc,
            } => NewlineInAttrs::new(attr_start_loc, newline_loc).log(),
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
            Self::UnexpectedHeading { loc } => UnexpectedHeading::new(loc).log(),
            Self::HeadingTooDeep { loc, level } => HeadingTooDeep::new(loc, level).log(),
            Self::TooManyQualifiers {
                loc,
                dot_locs: dot_loc,
            } => TooManyQualifiers::new(loc, dot_loc).log(),
            Self::EmptyQualifier { loc, qualifier_loc } => {
                EmptyQualifier::new(loc, qualifier_loc).log()
            }
        }
    }
}

impl Error for LexicalError {}

impl Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedChar { found, loc } => {
                write!(f, "unexpected character {:?} found at {}", found, loc)
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
            Self::NewlineInAttrs { attr_start_loc, .. } => {
                write!(f, "newline in attributes found at {}", attr_start_loc)
            }
            Self::NewlineInEmphDelimiter {
                newline_loc,
                expected,
                ..
            } => {
                write!(
                    f,
                    r#"newline in {} emphasis found at {}"#,
                    expected.quote(),
                    newline_loc
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
            Self::UnexpectedHeading { loc } => {
                write!(f, "unexpected heading at {loc}")
            }
            Self::HeadingTooDeep { loc, level } => {
                write!(f, "heading too deep at {loc} ({level} levels)")
            }
            Self::TooManyQualifiers { loc, dot_locs } => {
                write!(
                    f,
                    "extra dots found at {} in command name at {loc}",
                    dot_locs
                        .iter()
                        .map(|l| l.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Self::EmptyQualifier { loc, qualifier_loc } => {
                write!(
                    f,
                    "empty qualifier found at {qualifier_loc} in command name at {loc}",
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
