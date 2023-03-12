pub enum Version {
    V1_0,
}

impl Version {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1_0 => "v1.0",
        }
    }
}
