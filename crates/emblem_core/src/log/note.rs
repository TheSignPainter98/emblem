use crate::log::MessageType;
use crate::parser::Location;

#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    loc: Location,
    msg: String,
    msg_type: MessageType,
}

impl Note {
    fn new(msg_type: MessageType, loc: &Location, msg: impl Into<String>) -> Self {
        Self {
            loc: loc.clone(),
            msg: msg.into(),
            msg_type,
        }
    }

    pub fn error(loc: &Location, msg: impl Into<String>) -> Self {
        Self::new(MessageType::Error, loc, msg)
    }

    #[allow(dead_code)]
    pub fn warn(loc: &Location, msg: impl Into<String>) -> Self {
        Self::new(MessageType::Warning, loc, msg)
    }

    pub fn info(loc: &Location, msg: impl Into<String>) -> Self {
        Self::new(MessageType::Info, loc, msg)
    }

    #[allow(dead_code)]
    pub fn help(loc: &Location, msg: impl Into<String>) -> Self {
        Self::new(MessageType::Help, loc, msg)
    }

    pub fn loc(&self) -> &Location {
        &self.loc
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn msg_type(&self) -> MessageType {
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

    pub fn message_types(&self) -> Vec<MessageType> {
        vec![self.msg_type]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        log::MessageType,
        parser::{Location, Point},
        Context,
    };

    fn placeholder_loc() -> Location {
        let ctx = Context::test_new();
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
            MessageType::Error,
            Note::error(&placeholder_loc(), "foo").msg_type()
        );
        assert_eq!(
            MessageType::Warning,
            Note::warn(&placeholder_loc(), "foo").msg_type()
        );
        assert_eq!(
            MessageType::Info,
            Note::info(&placeholder_loc(), "foo").msg_type()
        );
        assert_eq!(
            MessageType::Help,
            Note::help(&placeholder_loc(), "foo").msg_type()
        );
    }
}
