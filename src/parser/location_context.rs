use derive_new::new;

#[derive(new)]
pub struct LocationContext<'i> {
    src: &'i str,
    starting_index: usize,
}

impl<'i> LocationContext<'i> {
    pub fn src(&self) -> &'i str {
        self.src
    }

    pub fn starting_index(&self) -> usize {
        self.starting_index
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn getters() {
        let src = "all your base are belong to us";
        let ctx = LocationContext::new(src, 12);

        assert_eq!(src, ctx.src());
        assert_eq!(12, ctx.starting_index());
    }
}
