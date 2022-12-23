pub mod parsed;
pub mod region;
pub mod text;

pub type ParsedAst<'file> = File<parsed::Content<'file>>;

#[derive(Debug)]
pub struct File<C> {
    pub pars: Vec<Par<C>>,
}

#[derive(Debug)]
pub struct Par<C> {
    pub content: Vec<C>,
}
