use typed_arena::Arena;

#[derive(Default)]
pub struct Context {
    files: Arena<File>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            files: Arena::new(),
        }
    }

    pub fn alloc_file(&mut self, name: String, content: String) -> &File {
        self.files.alloc(File { name, content })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct File {
    name: String,
    content: String,
}

impl File {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn alloc_file() {
        let mut ctx = Context::new();
        let name = "/usr/share/man/man1/gcc.1.gz".to_owned();
        let content = "hello, world".to_owned();

        let file = ctx.alloc_file(name.clone(), content.clone());
        assert_eq!(file.name(), name);
        assert_eq!(file.content(), content);
    }
}
