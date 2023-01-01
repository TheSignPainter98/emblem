pub mod parsed;
pub mod region;
mod text;
mod debug;

// use std::fmt::Display;

pub use text::Text;
#[cfg(test)]
pub use debug::AstDebug;

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
