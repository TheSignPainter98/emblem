use crate::{
    args::ExplainCmd,
    log::messages,
    log::messages::{
        NoSuchErrorCode,
    }
};
use std::error::Error;
use pager::Pager;

pub fn explain(cmd: ExplainCmd) -> Result<(), Box<dyn Error>> {
    match get_explanation(&cmd.id) {
        Ok(expl) => {
            Pager::with_default_pager("less").setup();
            println!("{}", expl);
        },
        Err(m) => alert!(m),
    }

    Ok(())
}

fn get_explanation<'a>(id: &'a str) -> Result<&'static str, NoSuchErrorCode<'a>> {
    if id.is_empty() {
        return Err(NoSuchErrorCode::new(id));
    }

    let msg = messages::messages().into_iter().filter(|msg| msg.id() == id).next();

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
