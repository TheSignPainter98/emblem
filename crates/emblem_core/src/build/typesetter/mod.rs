use std::error::Error;

use crate::{
    ast::parsed::ParsedFile,
    build::typesetter::doc::Doc,
    extensions::{Event, ExtensionState},
};

pub(crate) mod doc;

// TODO(kcza): typesettable file -> [fragment]

pub struct Typesetter<'em> {
    max_iters: Option<u32>,
    ext_state: &'em mut ExtensionState,
    #[allow(unused)]
    root: Doc<'em>,
}

impl<'em> Typesetter<'em> {
    pub fn new(ext_state: &'em mut ExtensionState, root: ParsedFile<'em>) -> Self {
        Self {
            max_iters: None,
            ext_state,
            root: Doc::from(root),
        }
    }

    pub fn set_max_iters(mut self, max_iters: u32) -> Self {
        self.max_iters = Some(max_iters);
        self
    }

    pub fn typeset(mut self) -> Result<(), Box<dyn Error>> {
        loop {
            self.iter()?;

            if !self.will_reiter() {
                break;
            }

            self.reset_reiter_request();
        }

        self.ext_state.handle(Event::Done)?;

        Ok(())
    }

    fn will_reiter(&self) -> bool {
        self.ext_state.reiter_requested()
            && self.ext_state.curr_iter() < self.max_iters.unwrap_or(u32::MAX)
    }

    fn reset_reiter_request(&self) {
        self.ext_state.reset_reiter_request();
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
