use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct TooManyDisambiguators<'i> {
    loc: Location<'i>,
    dot_locs: Vec<Location<'i>>,
}

impl<'i> Message<'i> for TooManyDisambiguators<'i> {
    fn id() -> &'static str {
        "E005"
    }

    fn log(self) -> Log<'i> {
        let msg = if self.dot_locs.len() > 1 {
            "extra dot in command name"
        } else {
            "extra dots in command name"
        };
        Log::error(msg).with_id(Self::id()).explainable().with_src({
            let mut src = Src::new(&self.loc);
            for (i, dot_loc) in self.dot_locs.iter().enumerate() {
                src = src.with_annotation(Note::error(
                    dot_loc,
                    if i == 0 { "found here" } else { "and here" },
                ));
            }
            src
        })
    }

    fn explain(&self) -> &'static str {
        concat!(
            "This error means that a command has been specified with too many disambiguators, that is, .pkg.pkg2.cmd is an invalid way of accessing 'cmd'. ",
            "To keep things concise, emblem only allows for a single disambiguator per command, for example, .pkg.cmd"
        )
    }
}
