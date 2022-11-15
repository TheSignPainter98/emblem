#[derive(Debug, Default, PartialEq, Eq, Ord, PartialOrd)]
struct IndentLevel {
    tabs: u32,
    spaces: u32,
}

const SPACES_PER_TAB: u32 = 4;

impl IndentLevel {
    fn level(&self) -> u32 {
        self.tabs + (self.spaces as f32 / SPACES_PER_TAB as f32).ceil() as u32
    }
}

impl From<&str> for IndentLevel {
    fn from(other: &str) -> Self {
        let mut asdf = Self { tabs: 0, spaces: 0 };

        for chr in other.chars() {
            match chr {
                ' ' => asdf.spaces += 1,
                '\t' => asdf.tabs += 1,
                _ => {}
            }
        }

        asdf
    }
}

#[cfg(test)]
mod test {
    mod indent_level {
        use super::super::*;
        #[test]
        fn default() {
            assert_eq!(IndentLevel { tabs: 0, spaces: 0 }, IndentLevel::default());
        }

        #[test]
        fn level() {
            assert_eq!(0, IndentLevel{ tabs: 0, spaces: 0}.level());
            assert_eq!(1, IndentLevel{ tabs: 0, spaces: 1}.level());
            assert_eq!(1, IndentLevel{ tabs: 1, spaces: 0}.level());
            assert_eq!(2, IndentLevel{ tabs: 1, spaces: 1}.level());
        }

        #[test]
        fn from() {
            assert_eq!(IndentLevel { tabs: 0, spaces: 0 }, IndentLevel::from(""));
            assert_eq!(IndentLevel { tabs: 0, spaces: 1 }, IndentLevel::from(" "));
            assert_eq!(IndentLevel { tabs: 1, spaces: 0 }, IndentLevel::from("\t"));
            assert_eq!(IndentLevel { tabs: 1, spaces: 1 }, IndentLevel::from(" \t"));
        }
    }
}
