mod debug;
pub mod parsed;
mod repr_loc;
mod text;

#[cfg(test)]
pub use debug::AstDebug;
pub use repr_loc::ReprLoc;
pub use text::Text;

#[derive(Debug)]
pub struct File<T> {
    pub pars: Vec<Par<T>>,
}

impl<T> From<Vec<Par<T>>> for File<T> {
    fn from(pars: Vec<Par<T>>) -> Self {
        Self { pars }
    }
}

#[derive(Debug)]
pub struct Par<T> {
    pub parts: Vec<T>,
}

impl<T> From<Vec<T>> for Par<T> {
    fn from(parts: Vec<T>) -> Self {
        Self { parts }
    }
}

impl<T> From<T> for Par<T> {
    fn from(part: T) -> Self {
        Self { parts: vec![part] }
    }
}

impl<T> Par<ParPart<T>> {
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty() || self.parts.iter().all(|part| part.is_empty())
    }
}

#[derive(Debug)]
pub enum ParPart<T> {
    Line(Vec<T>),
    Command(T),
}

impl<T> ParPart<T> {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Line(l) => l.is_empty(),
            Self::Command(_) => false,
        }
    }
}

#[cfg(test)]
impl<T> ParPart<T> {
    fn line(&self) -> Option<&[T]> {
        match self {
            Self::Line(l) => Some(l),
            Self::Command(_) => None,
        }
    }

    #[allow(unused)]
    fn command(&self) -> Option<&T> {
        match self {
            Self::Line(_) => None,
            Self::Command(c) => Some(c),
        }
    }
}

#[cfg(test)]
impl<T: AstDebug> AstDebug for File<T> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        buf.push("File".into());
        self.pars.test_fmt(buf);
    }
}

#[cfg(test)]
impl<T: AstDebug> AstDebug for Par<T> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        buf.push("Par".into());
        self.parts.test_fmt(buf);
    }
}

#[cfg(test)]
impl<T: AstDebug> AstDebug for ParPart<T> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        match self {
            Self::Line(l) => l.test_fmt(buf),
            Self::Command(s) => s.test_fmt(buf),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Dash {
    Hyphen,
    En,
    Em,
}

impl<T: AsRef<str>> From<T> for Dash {
    fn from(s: T) -> Self {
        #[cfg(test)]
        assert!(s.as_ref().chars().all(|c| c == '-'));

        match s.as_ref().len() {
            1 => Self::Hyphen,
            2 => Self::En,
            3 => Self::Em,
            _ => panic!(
                "Dash::from expected from 1 to 3 dashes: got {}",
                s.as_ref().len()
            ),
        }
    }
}

#[cfg(test)]
impl AstDebug for Dash {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        buf.push(
            match self {
                Self::Hyphen => "-",
                Self::En => "--",
                Self::Em => "---",
            }
            .into(),
        );
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Glue {
    Tight,
    Nbsp,
}

impl<T: AsRef<str>> From<T> for Glue {
    fn from(s: T) -> Self {
        #[cfg(test)]
        assert!(s.as_ref().chars().all(|c| c == '~'));

        match s.as_ref().len() {
            1 => Self::Tight,
            2 => Self::Nbsp,
            _ => panic!(
                "Glue::from expected from 1 to 2 tildes: got {}",
                s.as_ref().len()
            ),
        }
    }
}

#[cfg(test)]
impl AstDebug for Glue {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        buf.push(
            match self {
                Self::Tight => "~",
                Self::Nbsp => "~~",
            }
            .into(),
        );
    }
}
