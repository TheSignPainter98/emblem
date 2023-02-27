use crate::args::ArgPath;
use crate::context::Context;
use crate::log::messages::Message;
use crate::parser;
use crate::path::SearchResult;
use crate::Action;
use crate::Log;
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
    fn run<'em>(&self, ctx: &'em mut Context) -> Vec<Log<'em>> {
        let fname: SearchResult = match self.input.as_ref().try_into() {
            Ok(f) => f,
            Err(e) => return vec![Log::error(e.to_string())],
        };

        match parser::parse_file(ctx, fname) {
            Ok(d) => {
                println!("{d:?}");
                vec![]
            }
            Err(e) => vec![e.log()],
        }
    }
}
