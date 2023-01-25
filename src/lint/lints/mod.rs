mod command_naming;
mod excessive_args;

use super::Lints;

pub fn lints() -> Lints {
    macro_rules! lints {
        ($($lint:expr),* $(,)?) => {
            vec![
                $(Box::new($lint),)*
            ]
        }
    }

    lints![
        command_naming::CommandNaming::new(),
        excessive_args::ExcessiveArgs::new(),
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
        let lints = lints();
        let ids = lints.iter().map(|l| l.id()).collect::<Vec<_>>();

        let num_lints = lints.len();
        let num_unique_ids = ids.iter().collect::<HashSet<_>>().len();
        assert_eq!(num_lints, num_unique_ids);
    }
}
