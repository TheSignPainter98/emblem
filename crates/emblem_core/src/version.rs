use strum::EnumIter;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
pub enum Version {
    V1_0,
    V1_1,
}

impl Version {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1_0 => "1.0",
            Self::V1_1 => "1.1",
        }
    }

    pub(crate) fn latest() -> Self {
        Self::V1_0
    }
}
