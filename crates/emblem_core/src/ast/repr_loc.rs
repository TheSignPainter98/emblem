use crate::{
    ast::{
        parsed::{Content, Sugar},
        Par, ParPart,
    },
    parser::Location,
};

pub trait ReprLoc<'em> {
    fn repr_loc(&self) -> Location<'em>;
}

impl<'em> ReprLoc<'em> for Par<ParPart<Content<'em>>> {
    fn repr_loc(&self) -> Location<'em> {
        let parts = self
            .parts
            .iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        parts
            .first()
            .unwrap()
            .repr_loc()
            .span_to(&parts.last().unwrap().repr_loc())
    }
}

impl<'em> ReprLoc<'em> for ParPart<Content<'em>> {
    fn repr_loc(&self) -> Location<'em> {
        match self {
            Self::Line(l) => match &l[..] {
                [sole] => sole.repr_loc(),
                [f, .., l] => f.repr_loc().span_to(&l.repr_loc()),
                [] => panic!("internal error: empty line"),
            },
            Self::Command(Content::Command { invocation_loc, .. }) => invocation_loc.clone(),
            Self::Command(_) => {
                panic!("internal error: par-part command doesn't contain a command")
            }
        }
    }
}

impl<'em> ReprLoc<'em> for Content<'em> {
    fn repr_loc(&self) -> Location<'em> {
        match self {
            Self::Shebang { loc, .. }
            | Self::Command {
                invocation_loc: loc,
                ..
            }
            | Self::Word { loc, .. }
            | Self::Whitespace { loc, .. }
            | Self::Dash { loc, .. }
            | Self::Glue { loc, .. }
            | Self::SpiltGlue { loc, .. }
            | Self::Verbatim { loc, .. }
            | Self::Comment { loc, .. }
            | Self::MultiLineComment { loc, .. } => loc.clone(),
            Self::Sugar(sugar) => sugar.repr_loc(),
        }
    }
}

impl<'em> ReprLoc<'em> for Sugar<'em> {
    fn repr_loc(&self) -> Location<'em> {
        match self {
            Self::Italic { loc, .. }
            | Self::Bold { loc, .. }
            | Self::Monospace { loc, .. }
            | Self::Smallcaps { loc, .. }
            | Self::AlternateFace { loc, .. }
            | Self::Heading {
                invocation_loc: loc,
                ..
            }
            | Self::Mark { loc, .. }
            | Self::Reference { loc, .. } => loc.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ast::parsed::ParsedFile, parser};

    fn parse<'i>(name: &'i str, src: &'i str) -> ParsedFile<'i> {
        parser::parse(name, src).unwrap()
    }

    #[test]
    fn par() {
        let file = parse("par", "have\nfun");
        assert_eq!(
            "par:1:1-2:4",
            file.pars.first().unwrap().repr_loc().to_string()
        );
    }

    #[test]
    fn par_part_command() {
        let remainder_file = parse(
            "par-part-command",
            ".dont{get attached}: to somebody you could lose",
        );
        assert_eq!(
            "par-part-command:1:1-5",
            remainder_file
                .pars
                .first()
                .unwrap()
                .parts
                .first()
                .unwrap()
                .repr_loc()
                .to_string()
        );

        let trailer_file = parse("par-part-command", ".wear{your heart}:\n\ton your cheek");
        assert_eq!(
            "par-part-command:1:1-5",
            trailer_file.pars[0].parts[0].repr_loc().to_string()
        );
    }

    #[test]
    fn content() {
        let tests = [
            ("command", "1:5-8", 2, "XXX .foo{asdf}{fdas} XXX"),
            ("word", "1:5-7", 2, "XXX foo XXX"),
            ("whitespace", "1:4-9", 1, "XXX \t XXX"),
            ("dash", "1:5-7", 2, "XXX --- XXX"),
            ("glue", "1:4-4", 1, "XXX~XXX"),
            ("spilt-glue", "1:4-6", 1, "XXX ~ XXX"),
            ("verbatim", "1:5-10", 2, "XXX !verb! XXX"),
            ("comment", "1:5-15", 2, "XXX // hfjkdasl"),
            ("multi-line-comment", "1:5-18", 2, "XXX /* hfjkdasl */ XXX"),
        ];

        for (name, loc, idx, src) in tests {
            let repr_loc = parse(name, src).pars[0].parts[0].line().unwrap()[idx]
                .repr_loc()
                .to_string();
            assert_eq!(
                format!("{name}:{loc}"),
                repr_loc,
                "incorrect location for {name}={src}: {:#?}",
                parse(name, src),
            );
        }
    }

    #[test]
    fn sugar() {
        let tests = [
            ("italic", "1:1-5", "_foo_"),
            ("bold", "1:1-7", "**foo**"),
            ("monospace", "1:1-5", "`foo`"),
            ("small-caps", "1:1-5", "=foo="),
            ("alternate-face", "1:1-7", "==foo=="),
            ("heading-1", "1:1-1", "# foo"),
            ("heading-2", "1:1-2", "## foo"),
            ("heading-3", "1:1-3", "### foo"),
            ("heading-4", "1:1-4", "#### foo"),
            ("heading-5", "1:1-5", "##### foo"),
            ("heading-6", "1:1-6", "###### foo"),
            ("heading-1-plus", "1:1-2", "#+ foo"),
            ("heading-2-plus", "1:1-3", "##+ foo"),
            ("heading-3-plus", "1:1-4", "###+ foo"),
            ("heading-4-plus", "1:1-5", "####+ foo"),
            ("heading-5-plus", "1:1-6", "#####+ foo"),
            ("heading-6-plus", "1:1-7", "######+ foo"),
        ];

        for (name, loc, src) in tests {
            assert_eq!(
                format!("{name}:{loc}"),
                parse(name, src)
                    .pars
                    .first()
                    .unwrap()
                    .parts
                    .first()
                    .unwrap()
                    .repr_loc()
                    .to_string()
            );
        }
    }
}
