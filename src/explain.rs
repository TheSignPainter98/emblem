use crate::{
    args::ExplainCmd,
    log::messages::{self, NoSuchErrorCode},
};
use std::error::Error;

pub fn explain(cmd: ExplainCmd) -> Result<(), Box<dyn Error>> {
    match get_explanation(&cmd.id) {
        Ok(expl) => println!("{}", expl),
        Err(m) => alert!(m),
    }

    Ok(())
}

fn get_explanation(id: &str) -> Result<&'static str, NoSuchErrorCode<'_>> {
    if id.is_empty() {
        return Err(NoSuchErrorCode::new(id));
    }

    let msg = messages::messages().into_iter().find(|msg| msg.id() == id);

    match msg {
        None => Err(NoSuchErrorCode::new(id)),
        Some(msg) => Ok(msg.explanation()),
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn get_explanation() {
        assert!(super::get_explanation("E001").is_ok());
    }
}
