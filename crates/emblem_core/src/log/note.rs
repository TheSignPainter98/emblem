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
