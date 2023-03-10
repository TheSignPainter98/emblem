use crate::log::Note;
use crate::parser::Location;

#[cfg(test)]
use annotate_snippets::snippet::AnnotationType;

#[derive(Clone, Debug, PartialEq)]
pub struct Src<'i> {
    loc: Location<'i>,
    annotations: Vec<Note<'i>>,
}

impl<'i> Src<'i> {
    pub fn new(loc: &Location<'i>) -> Self {
        Self {
            loc: loc.clone(),
            annotations: Vec::new(),
        }
    }

    pub fn with_annotation(mut self, note: Note<'i>) -> Self {
        self.annotations.push(note);
        self
    }

    pub fn loc(&self) -> &Location<'i> {
        &self.loc
    }

    pub fn annotations(&self) -> &Vec<Note<'i>> {
        &self.annotations
    }
}

#[cfg(test)]
impl Src<'_> {
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
