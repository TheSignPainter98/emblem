pub(crate) mod typesetter;

use crate::args::ArgPath;
use crate::context::Context;
use crate::parser;
use crate::path::SearchResult;
use crate::Action;
use crate::Result;
use derive_new::new;

#[derive(new)]
pub struct Builder {
    input: ArgPath,

    #[allow(unused)]
    output_stem: ArgPath,

    #[allow(unused)]
    output_driver: Option<String>,
}

impl Action for Builder {
    type Response = Option<Vec<(ArgPath, String)>>;

    fn run(&self, ctx: &mut Context) -> Result<Self::Response> {
        let fname: SearchResult = self.input.as_ref().try_into()?;
        let root = parser::parse_file(ctx, fname)?;
        ctx.typesetter().typeset(root)?;
        Ok(None)
    }
}
