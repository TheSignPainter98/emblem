use crate::parser::Location;
use annotate_snippets::snippet::AnnotationType;

#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    loc: Location,
    msg: String,
    msg_type: AnnotationType,
}

impl Note {
    fn new<S: Into<String>>(msg_type: AnnotationType, loc: &Location, msg: S) -> Self {
        Self {
            loc: loc.clone(),
            msg: msg.into(),
            msg_type,
        }
    }

    pub fn error<S: Into<String>>(loc: &Location, msg: S) -> Self {
        Self::new(AnnotationType::Error, loc, msg)
    }

    #[allow(dead_code)]
    pub fn warn<S: Into<String>>(loc: &Location, msg: S) -> Self {
        Self::new(AnnotationType::Warning, loc, msg)
    }

    pub fn info<S: Into<String>>(loc: &Location, msg: S) -> Self {
        Self::new(AnnotationType::Info, loc, msg)
    }

    #[allow(dead_code)]
    pub fn help<S: Into<String>>(loc: &Location, msg: S) -> Self {
        Self::new(AnnotationType::Help, loc, msg)
    }

    pub fn loc(&self) -> &Location {
        &self.loc
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn msg_type(&self) -> AnnotationType {
        self.msg_type
    }
}

#[cfg(test)]
impl Note {
    pub fn text(&self) -> Vec<&str> {
        vec![&self.msg]
    }

    pub fn annotation_text(&self) -> Vec<String> {
        vec![format!("{}: {}", self.loc, self.msg)]
    }

    pub fn log_levels(&self) -> Vec<AnnotationType> {
        vec![self.msg_type]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        parser::{Location, Point},
        Context,
    };

    fn placeholder_loc() -> Location {
        let ctx = Context::new();
        let p = Point::at_start_of(
            ctx.alloc_file_name("main.em"),
            ctx.alloc_file_content("hello, world!"),
        );
        let shifted = p.clone().shift("hello");
        Location::new(&p, &shifted)
    }

    #[test]
    pub fn loc() {
        let loc = placeholder_loc();

        assert_eq!(&loc, Note::error(&loc, "foo").loc());
        assert_eq!(&loc, Note::warn(&loc, "foo").loc());
        assert_eq!(&loc, Note::info(&loc, "foo").loc());
        assert_eq!(&loc, Note::help(&loc, "foo").loc());
    }

    #[test]
    pub fn msg() {
        assert_eq!("sup", Note::error(&placeholder_loc(), "sup").msg());
    }

    #[test]
    pub fn msg_type() {
        assert_eq!(
            AnnotationType::Error,
            Note::error(&placeholder_loc(), "foo").msg_type()
        );
        assert_eq!(
            AnnotationType::Warning,
            Note::warn(&placeholder_loc(), "foo").msg_type()
        );
        assert_eq!(
            AnnotationType::Info,
            Note::info(&placeholder_loc(), "foo").msg_type()
        );
        assert_eq!(
            AnnotationType::Help,
            Note::help(&placeholder_loc(), "foo").msg_type()
        );
    }
}
