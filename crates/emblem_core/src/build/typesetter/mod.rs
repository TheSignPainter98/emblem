use std::error::Error;

use crate::{
    ast::parsed::ParsedFile,
    build::typesetter::doc::Doc,
    extensions::{Event, ExtensionState},
};

pub(crate) mod doc;

// TODO(kcza): typesettable file -> [fragment]

pub struct Typesetter<'ext> {
    max_iters: Option<u32>,
    ext_state: &'ext mut ExtensionState,
}

impl<'ext> Typesetter<'ext> {
    pub fn new(ext_state: &'ext mut ExtensionState) -> Self {
        Self {
            max_iters: None,
            ext_state,
        }
    }

    pub fn set_max_iters(mut self, max_iters: u32) -> Self {
        self.max_iters = Some(max_iters);
        self
    }

    pub fn typeset(mut self, parsed_doc: ParsedFile<'_>) -> Result<(), Box<dyn Error>> {
        let doc = Doc::from(parsed_doc);
        println!("{doc:#?}");

        while self.reiter_required() {
            self.iter()?;
        }

        self.ext_state.handle(Event::Done)?;

        Ok(())
    }

    fn reiter_required(&self) -> bool {
        self.ext_state.reiter_requested()
            && self.ext_state.curr_iter() < self.max_iters.unwrap_or(u32::MAX)
    }

    fn iter(&mut self) -> Result<(), Box<dyn Error>> {
        self.ext_state.increment_iter_count();
        self.ext_state.handle(Event::IterStart)?;

        println!(
            "Doing iteration {} of {}",
            self.ext_state.curr_iter(),
            self.max_iters.unwrap_or(u32::MAX)
        );

        self.ext_state.handle(Event::IterEnd)?;

        Ok(())
    }
}
