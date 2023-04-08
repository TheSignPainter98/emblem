pub(crate) mod typesetter;

use crate::args::ArgPath;
use crate::context::Context;
use crate::log::messages::Message;
use crate::parser;
use crate::path::SearchResult;
use crate::Action;
use crate::EmblemResult;
use crate::ExtensionStateBuilder;
use crate::Log;
use derive_new::new;

use self::typesetter::Typesetter;

#[derive(new)]
pub struct Builder {
    input: ArgPath,

    #[allow(unused)]
    output_stem: ArgPath,

    #[allow(unused)]
    output_driver: Option<String>,

    max_iters: u32,

    ext_state_builder: ExtensionStateBuilder,
}

impl Action for Builder {
    type Response = Option<Vec<(ArgPath, String)>>;

    fn run<'ctx>(&self, ctx: &'ctx mut Context) -> EmblemResult<'ctx, Self::Response> {
        let fname: SearchResult = match self.input.as_ref().try_into() {
            Ok(f) => f,
            Err(e) => return EmblemResult::new(vec![Log::error(e.to_string())], None),
        };

        let doc = match parser::parse_file(ctx, fname) {
            Ok(d) => d,
            Err(e) => return EmblemResult::new(vec![e.log()], None),
        };

        let mut ext_state = self.ext_state_builder.build().unwrap(); // TODO(kcza): remove this unwrap!

        let typesetter = Typesetter::new(&mut ext_state).set_max_iters(self.max_iters);
        typesetter.typeset(doc).unwrap();

        EmblemResult::new(vec![], Some(vec![]))
    }
}
