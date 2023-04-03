use std::error::Error;

use crate::{
    ast::parsed::ParsedFile,
    build::typesetter::doc::Doc,
    extensions::{Event, ExtensionState},
    Context,
};

pub(crate) mod doc;

// TODO(kcza): typesettable file -> [fragment]

pub struct Typesetter<'ext> {
    curr_iter: u32,
    max_iters: Option<u32>,
    reiter_requested: bool,
    ext_state: &'ext ExtensionState,
}

impl<'ext> Typesetter<'ext> {
    pub fn new(ext_state: &'ext ExtensionState) -> Self {
        Self {
            curr_iter: 0,
            max_iters: None,
            reiter_requested: true,
            ext_state,
        }
    }

    pub fn set_max_iters(&mut self, max_iters: u32) {
        self.max_iters = Some(max_iters);
    }

    pub fn typeset(mut self, parsed_doc: ParsedFile<'_>) -> Result<(), Box<dyn Error>> {
        let doc = Doc::from(parsed_doc);
        println!("{doc:#?}");

        while self.curr_iter < self.max_iters.unwrap_or(u32::MAX) && self.reiter_requested {
            self.ext_state.handle(Event::IterStart)?;

            self.curr_iter += 1;
            self.reiter_requested = false;

            println!(
                "Doing iteration {} of {}",
                self.curr_iter,
                self.max_iters.unwrap_or(u32::MAX)
            );

            self.ext_state.handle(Event::IterEnd)?;
        }

        self.ext_state.handle(Event::Done)?;

        Ok(())
    }
}
