mod debug;
pub mod parsed;
pub mod region;
mod text;

// use std::fmt::Display;

#[cfg(test)]
pub use debug::AstDebug;
pub use text::Text;

pub type ParsedAst<'file> = File<parsed::Content<'file>>;

#[derive(Debug)]
pub struct File<C> {
    pub pars: Vec<Par<C>>,
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
pub struct Par<C> {
    pub lines: Vec<Line<C>>,
}

impl<C> From<Vec<Line<C>>> for Par<C> {
    fn from(lines: Vec<Line<C>>) -> Self {
        Self { lines }
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
pub struct Line<C> {
    pub content: Vec<C>,
}

impl<C> From<Vec<C>> for Line<C> {
    fn from(content: Vec<C>) -> Self {
        Self { content }
    }
}

#[cfg(test)]
impl<C: AstDebug> AstDebug for File<C> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        buf.push("File".into());
        self.pars.test_fmt(buf);
    }
}

#[cfg(test)]
impl<C: AstDebug> AstDebug for Par<C> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        buf.push("Par".into());
        self.lines.test_fmt(buf);
    }
}

#[cfg(test)]
impl<C: AstDebug> AstDebug for Line<C> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        self.content.test_fmt(buf);
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
        buf.push(match self {
            Self::Hyphen => "-",
            Self::En => "--",
            Self::Em => "---",
        }.into());
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
        buf.push(match self {
            Self::Tight => "~",
            Self::Nbsp => "~~",
        }.into());
    }
}
