use crate::FileContentSlice;

pub(crate) enum VerbatimBlockContent {
    Line(FileContentSlice),
    Lines(Vec<VerbatimBlockContent>),
}

impl VerbatimBlockContent {
    pub(crate) fn flatten(self) -> Vec<FileContentSlice> {
        if let Self::Line(line) = self {
            return vec![line];
        }

        let mut content = vec![];
        self.flatten_into(&mut content);
        content
    }

    fn flatten_into(self, content: &mut Vec<FileContentSlice>) {
        match self {
            Self::Line(line) => content.push(line),
            Self::Lines(lines) => lines
                .into_iter()
                .for_each(|line| line.flatten_into(content)),
        }
    }
}
