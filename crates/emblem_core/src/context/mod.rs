mod dependency;

use crate::Version;
pub use dependency::{DependencyName, Dependency, DependencyVersion};
use typed_arena::Arena;

#[derive(Default)]
pub struct Context<'m> {
    files: Arena<File>,
    doc_info: DocInfo<'m>,
    lua_info: LuaInfo<'m>,
    dependencies: Option<Vec<(DependencyName<'m>, Dependency<'m>)>>,
}

impl<'m> Context<'m> {
    pub fn new() -> Self {
        Self {
            files: Arena::new(),
            doc_info: Default::default(),
            lua_info: Default::default(),
            dependencies: None,
        }
    }

    pub fn alloc_file(&mut self, name: String, content: String) -> &File {
        self.files.alloc(File { name, content })
    }

    pub fn doc_info(&self) -> &DocInfo<'m> {
        &self.doc_info
    }

    pub fn doc_info_mut(&mut self) -> &mut DocInfo<'m> {
        &mut self.doc_info
    }

    pub fn lua_info(&self) -> &LuaInfo<'m> {
        &self.lua_info
    }

    pub fn lua_info_mut(&mut self) -> &mut LuaInfo<'m> {
        &mut self.lua_info
    }

    pub fn set_dependencies(&mut self, dependencies: Vec<(DependencyName<'m>, Dependency<'m>)>) {
        self.dependencies = Some(dependencies);
    }

    pub fn dependencies(&self) -> &Option<Vec<(DependencyName<'m>, Dependency<'m>)>> {
        &self.dependencies
    }

    pub fn dependencies_mut(&mut self) -> Option<&mut Vec<(DependencyName<'m>, Dependency<'m>)>> {
        match self.dependencies {
            Some(ref mut d) => Some(d),
            None => None,
        }
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

#[derive(Debug, Default)]
pub struct DocInfo<'m> {
    name: Option<&'m str>,
    emblem_version: Option<Version>,
    authors: Option<Vec<&'m str>>,
    keywords: Option<Vec<&'m str>>,
}

impl<'m> DocInfo<'m> {
    pub fn set_name(&mut self, name: &'m str) {
        self.name = Some(name);
    }

    pub fn name(&self) -> Option<&str> {
        match self.name.as_ref() {
            None => None,
            Some(n) => Some(n),
        }
    }

    pub fn set_emblem_version(&mut self, emblem_version: Version) {
        self.emblem_version = Some(emblem_version);
    }

    pub fn emblem_version(&self) -> &Option<Version> {
        &self.emblem_version
    }

    pub fn set_authors(&mut self, authors: Vec<&'m str>) {
        self.authors = Some(authors);
    }

    pub fn authors(&self) -> &Option<Vec<&'m str>> {
        &self.authors
    }

    pub fn set_keywords(&mut self, keywords: Vec<&'m str>) {
        self.keywords = Some(keywords);
    }

    pub fn keywords(&self) -> &Option<Vec<&'m str>> {
        &self.keywords
    }
}

#[derive(Debug, Default)]
pub struct LuaInfo<'m> {
    sandbox: SandboxLevel,
    max_mem: MemoryLimit,
    general_args: Option<Vec<(&'m str, &'m str)>>,
    dependencies: Option<Vec<(&'m str, Dependency<'m>)>>,
}

impl<'m> LuaInfo<'m> {
    pub fn set_sandbox(&mut self, sandbox: SandboxLevel) {
        self.sandbox = sandbox;
    }

    pub fn sandbox(&self) -> SandboxLevel {
        self.sandbox
    }

    pub fn set_max_mem(&mut self, max_mem: MemoryLimit) {
        self.max_mem = max_mem;
    }

    pub fn max_mem(&self) -> MemoryLimit {
        self.max_mem
    }

    pub fn set_dependencies(&mut self, dependencies: Vec<(&'m str, Dependency<'m>)>) {
        self.dependencies = Some(dependencies);
    }

    pub fn dependencies(&self) -> &Option<Vec<(&'m str, Dependency<'m>)>> {
        &self.dependencies
    }

    pub fn set_general_args(&mut self, general_args: Vec<(&'m str, &'m str)>) {
        self.general_args = Some(general_args);
    }

    pub fn general_args(&self) -> &Option<Vec<(&'m str, &'m str)>> {
        &self.general_args
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub enum SandboxLevel {
    Unrestricted,
    #[default]
    Standard,
    Strict,
}

#[derive(Copy, Clone, Debug, Default)]
pub enum MemoryLimit {
    Limited(usize),
    #[default]
    Unlimited,
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
