mod command_naming;
mod empty_attrs;
mod num_args;
mod num_attrs;

use super::Lints;

pub fn lints<'i>() -> Lints<'i> {
    macro_rules! lints {
        ($($lint:expr),* $(,)?) => {
            vec![
                $(Box::new($lint),)*
            ]
        }
    }

    lints![
        command_naming::CommandNaming::new(),
        empty_attrs::EmptyAttrs::new(),
        num_args::NumArgs::new(),
        num_attrs::NumAttrs::new(),
    ]
}

#[cfg(test)]
mod test {
    use super::*;
    use lazy_static::lazy_static;
    use regex::Regex;
    use std::collections::HashSet;

    #[test]
    fn ids() {
        lazy_static! {
            static ref VALID_ID: Regex = Regex::new(r"^[a-z-]+$").unwrap();
        }

        let lints = lints();
        let ids = lints.iter().map(|l| l.id()).collect::<Vec<_>>();

        for id in &ids {
            if !VALID_ID.is_match(id) {
                panic!("IDs should be lowercase with dashes: got {}", id);
            }
        }
    }

    #[test]
    fn unique_ids() {
        let mut ids = HashSet::new();
        for lint in lints() {
            assert!(ids.insert(lint.id()), "id {:?} is not unique", lint.id());
        }
    }
}
