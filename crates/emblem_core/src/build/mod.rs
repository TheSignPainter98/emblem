mod typesetter;

use crate::args::ArgPath;
use crate::context::Context;
use crate::log::messages::Message;
use crate::parser;
use crate::path::SearchResult;
use crate::Action;
use crate::EmblemResult;
use crate::Log;
use derive_new::new;

use self::typesetter::typeset;

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

    fn run<'ctx>(&self, ctx: &'ctx mut Context) -> EmblemResult<'ctx, Self::Response> {
        let fname: SearchResult = match self.input.as_ref().try_into() {
            Ok(f) => f,
            Err(e) => return EmblemResult::new(vec![Log::error(e.to_string())], None),
        };

        let (logs, ret) = match parser::parse_file(ctx, fname) {
            Ok(d) => {
                typeset(d).unwrap();
                (vec![], Some(vec![]))
            }
            Err(e) => (vec![e.log()], None),
        };

        EmblemResult::new(logs, ret)
    }
}
