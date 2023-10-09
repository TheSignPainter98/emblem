use crate::{
    context::Context,
    log::{
        messages::{self /* Message */},
        LogId,
    },
    Action, Error, Result,
};
use derive_new::new;

#[derive(new)]
pub struct Explainer {
    id: LogId,
}

impl Action for Explainer {
    type Response = ();

    fn run(&self, _: &mut Context) -> Result<Self::Response> {
        if let Some(e) = self.get_explanation() {
            print!("{e}");
            Ok(())
        } else {
            Err(Error::no_such_error_code(self.id.clone()))
        }
    }
}

impl Explainer {
    fn get_explanation(&self) -> Option<&'static str> {
        if !self.id.is_defined() {
            return None;
        }

        messages::messages()
            .into_iter()
            .find(|msg| msg.id() == &self.id)
            .map(|msg| msg.explanation())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_explanation() {
        let explainer = Explainer::new("E001".into());
        assert!(explainer.get_explanation().is_some());
    }
}
