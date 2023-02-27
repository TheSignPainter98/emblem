use crate::{
    context::Context,
    log::messages::{self, Message, NoSuchErrorCode},
    Action, Log,
};
use derive_new::new;

#[derive(new)]
pub struct Explainer {
    id: String,
}

impl Action for Explainer {
    fn run<'em>(&self, _: &'em mut Context) -> Vec<Log<'em>> {
        match self.get_explanation() {
            Some(expl) => {
                println!("{}", expl);
                vec![]
            }
            None => vec![NoSuchErrorCode::new(self.id.clone()).log()],
        }
    }
}

impl Explainer {
    fn get_explanation(&self) -> Option<&'static str> {
        if self.id.is_empty() {
            return None;
        }

        messages::messages()
            .into_iter()
            .find(|msg| msg.id() == self.id)
            .map(|msg| msg.explanation())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_explanation() {
        let explainer = Explainer::new("E001".to_owned());
        assert!(explainer.get_explanation().is_some());
    }
}
