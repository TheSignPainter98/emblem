use strum::EnumIter;

use crate::log::MessageType;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
pub enum Verbosity {
    #[default]
    Terse,
    Verbose,
    Debug,
}

impl Verbosity {
    pub fn permits_printing(&self, msg_type: MessageType) -> bool {
        match (self, msg_type) {
            (Self::Terse, MessageType::Error) | (Self::Terse, MessageType::Warning) => true,
            (Self::Terse, _) => false,
            _ => true,
        }
    }
}
