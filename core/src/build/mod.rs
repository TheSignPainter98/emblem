use crate::args::ArgPath;
use crate::context::Context;
use crate::log::messages::Message;
use crate::parser;
use crate::path::SearchResult;
use crate::Action;
use crate::ActionResult;
use crate::EmblemResult;
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
    fn run<'ctx>(&self, ctx: &'ctx mut Context) -> EmblemResult<'ctx> {
        let fname: SearchResult = match self.input.as_ref().try_into() {
            Ok(f) => f,
            Err(e) => {
                return EmblemResult::new(
                    vec![Log::error(e.to_string())],
                    ActionResult::Build { output_files: None },
                )
            }
        };

        let logs = match parser::parse_file(ctx, fname) {
            Ok(d) => {
                println!("{d:?}");
                vec![]
            }
            Err(e) => vec![e.log()],
        };

        EmblemResult::new(logs, ActionResult::Build { output_files: None })
    }
}
