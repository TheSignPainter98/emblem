use std::fmt::Display;

// use crate::ast::region::Region;

#[derive(Clone, Debug)]
pub struct Problem {
    id: &'static str,
    // loc: Region<'i>,
    reason: String,
}

impl Problem {
    pub fn new(id: &'static str, reason: impl Into<String>) -> Self {
        Self {
            id,
            reason: reason.into(),
        }
    }

    // pub fn loc(mut self, loc: &'i Region<'i>) -> Self {
    //     &self.loc
    //     self
    // }
}

impl Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]:\n\t{}\n",
            self.id,
            self.reason,
            // self.hint.as_ref().unwrap_or_default(),
            // self.fix.as_ref().unwrap_or_default(),
        )
    }
}
