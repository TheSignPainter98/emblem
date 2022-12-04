pub struct Ast<C> {
    root: File<C>,
}

pub struct File<C> {
    name: String,
    pars: Vec<Par<C>>,
}

pub struct Par<C> {
    content: Vec<C>,
}
