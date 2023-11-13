use crate::log::Note;
use crate::parser::Location;

#[cfg(test)]
use annotate_snippets::snippet::AnnotationType;

#[derive(Clone, Debug, PartialEq)]
pub struct Src {
    loc: Location,
    annotations: Vec<Note>,
}

impl Src {
    pub fn new(loc: &Location) -> Self {
        Self {
            loc: loc.clone(),
            annotations: Vec::new(),
        }
    }

    pub fn loc(&self) -> &Location {
        &self.loc
    }

    pub fn with_annotation(mut self, note: Note) -> Self {
        self.annotations.push(note);
        self
    }

    pub fn annotations(&self) -> &Vec<Note> {
        &self.annotations
    }
}

#[cfg(test)]
impl Src {
    pub fn text(&self) -> Vec<&str> {
        self.annotations.iter().flat_map(|a| a.text()).collect()
    }

    pub fn annotation_text(&self) -> Vec<String> {
        self.annotations
            .iter()
            .flat_map(|a| a.annotation_text())
            .collect()
    }

    pub fn log_levels(&self) -> Vec<AnnotationType> {
        self.annotations
            .iter()
            .flat_map(|a| a.log_levels())
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        parser::{Location, Point},
        Context,
    };

    #[test]
    fn loc() {
        let ctx = Context::test_new();
        let p = Point::at_start_of(
            ctx.alloc_file_name("main.em"),
            ctx.alloc_file_content("1111111111111"),
        );
        let shifted = p.clone().shift("1111111111111");
        let loc = Location::new(&p, &shifted);

        assert_eq!(&loc, Src::new(&loc).loc());
    }

    #[test]
    fn annotations() {
        let ctx = Context::test_new();
        let start = Point::at_start_of(
            ctx.alloc_file_name("main.em"),
            ctx.alloc_file_content("111111222222"),
        );
        let mid = start.clone().shift("111111");
        let end = mid.clone().shift("222222");

        let mut src = Src::new(&Location::new(&start, &end));
        let annotations = [
            Note::error(&Location::new(&start, &mid), "foo"),
            Note::error(&Location::new(&mid, &end), "foo"),
        ];
        for annotation in &annotations {
            src = src.with_annotation(annotation.clone());
        }

        assert_eq!(annotations, src.annotations().as_slice());
    }
}
