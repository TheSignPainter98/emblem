use annotate_snippets::snippet::AnnotationType;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verbosity {
    #[default]
    Terse,
    Verbose,
    Debug,
}

impl Verbosity {
    pub fn permits_printing(&self, msg_type: AnnotationType) -> bool {
        match (self, msg_type) {
            (Self::Terse, AnnotationType::Error) | (Self::Terse, AnnotationType::Warning) => true,
            (Self::Terse, _) => false,
            _ => true,
        }
    }
}
