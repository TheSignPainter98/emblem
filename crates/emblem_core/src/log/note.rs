use crate::parser::Location;
use annotate_snippets::snippet::AnnotationType;

#[derive(Clone, Debug, PartialEq)]
pub struct Note<'i> {
    loc: Location<'i>,
    msg: String,
    msg_type: AnnotationType,
}

impl<'i> Note<'i> {
    fn new<S: Into<String>>(msg_type: AnnotationType, loc: &Location<'i>, msg: S) -> Self {
        Self {
            loc: loc.clone(),
            msg: msg.into(),
            msg_type,
        }
    }

    pub fn error<S: Into<String>>(loc: &Location<'i>, msg: S) -> Self {
        Self::new(AnnotationType::Error, loc, msg)
    }

    #[allow(dead_code)]
    pub fn warn<S: Into<String>>(loc: &Location<'i>, msg: S) -> Self {
        Self::new(AnnotationType::Warning, loc, msg)
    }

    pub fn info<S: Into<String>>(loc: &Location<'i>, msg: S) -> Self {
        Self::new(AnnotationType::Info, loc, msg)
    }

    #[allow(dead_code)]
    pub fn help<S: Into<String>>(loc: &Location<'i>, msg: S) -> Self {
        Self::new(AnnotationType::Help, loc, msg)
    }

    pub fn loc(&self) -> &Location<'i> {
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
impl Note<'_> {
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
        FileName,
    };

    fn dummy_loc() -> Location<'static> {
        let p = Point::new(FileName::new("main.em"), "hello, world!");
        let shifted = p.clone().shift("hello");
        Location::new(&p, &shifted)
    }

    #[test]
    pub fn loc() {
        let loc = dummy_loc();

        assert_eq!(&loc, Note::error(&loc, "foo").loc());
        assert_eq!(&loc, Note::warn(&loc, "foo").loc());
        assert_eq!(&loc, Note::info(&loc, "foo").loc());
        assert_eq!(&loc, Note::help(&loc, "foo").loc());
    }

    #[test]
    pub fn msg() {
        assert_eq!("sup", Note::error(&dummy_loc(), "sup").msg());
    }

    #[test]
    pub fn msg_type() {
        assert_eq!(
            AnnotationType::Error,
            Note::error(&dummy_loc(), "foo").msg_type()
        );
        assert_eq!(
            AnnotationType::Warning,
            Note::warn(&dummy_loc(), "foo").msg_type()
        );
        assert_eq!(
            AnnotationType::Info,
            Note::info(&dummy_loc(), "foo").msg_type()
        );
        assert_eq!(
            AnnotationType::Help,
            Note::help(&dummy_loc(), "foo").msg_type()
        );
    }
}
