mod debug;
pub mod parsed;
pub mod region;
mod text;

// use std::fmt::Display;

#[cfg(test)]
pub use debug::AstDebug;
pub use text::Text;

#[derive(Debug)]
pub struct File<T> {
    pub pars: Vec<Par<T>>,
}

// impl<C:Display> Display for File<C> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "File {{")?;
//         for (i, par) in self.pars.iter().enumerate() {
//             if i > 0 {
//                 write!(f, ",")?;
//             }
//             par.fmt(f)?;
//         }
//         write!(f, "}}")
//     }
// }

#[derive(Debug)]
pub struct Par<T> {
    pub parts: Vec<T>,
}

impl<T> From<Vec<T>> for Par<T> {
    fn from(lines: Vec<T>) -> Self {
        Self { parts: lines }
    }
}

impl<T> From<T> for Par<T> {
    fn from(line: T) -> Self {
        Self { parts: vec![line] }
    }
}

// impl<C:Display> Display for Par<C> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Pars {{")?;
//         for (i, c) in self.lines.iter().enumerate() {
//             if i > 0 {
//                 write!(f, ",")?;
//             }
//             c.fmt(f)?;
//         }
//         write!(f, "}}")
//     }
// }

#[derive(Debug)]
pub enum ParPart<T> {
    Line(Vec<T>),
    Command(T),
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

#[derive(Debug)]
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

#[derive(Debug)]
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
