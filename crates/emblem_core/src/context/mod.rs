mod module;

use crate::Version;
pub use module::{Module, ModuleName, ModuleVersion};
use typed_arena::Arena;

#[derive(Default)]
pub struct Context<'m> {
    files: Arena<File>,
    doc_info: DocInfo<'m>,
    lua_info: LuaInfo<'m>,
    modules: Option<Vec<(ModuleName<'m>, Module<'m>)>>,
}

impl<'m> Context<'m> {
    pub fn new() -> Self {
        Self {
            files: Arena::new(),
            doc_info: Default::default(),
            lua_info: Default::default(),
            modules: None,
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

    pub fn set_modules(&mut self, modules: Vec<(ModuleName<'m>, Module<'m>)>) {
        self.modules = Some(modules);
    }

    pub fn modules(&self) -> &Option<Vec<(ModuleName<'m>, Module<'m>)>> {
        &self.modules
    }

    pub fn modules_mut(&mut self) -> Option<&mut Vec<(ModuleName<'m>, Module<'m>)>> {
        self.modules.as_mut()
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
    max_mem: ResourceLimit,
    general_args: Option<Vec<(&'m str, &'m str)>>,
    modules: Option<Vec<(&'m str, Module<'m>)>>,
}

impl<'m> LuaInfo<'m> {
    pub fn set_sandbox(&mut self, sandbox: SandboxLevel) {
        self.sandbox = sandbox;
    }

    pub fn sandbox(&self) -> SandboxLevel {
        self.sandbox
    }

    pub fn set_max_mem(&mut self, max_mem: ResourceLimit) {
        self.max_mem = max_mem;
    }

    pub fn max_mem(&self) -> ResourceLimit {
        self.max_mem
    }

    pub fn set_modules(&mut self, modules: Vec<(&'m str, Module<'m>)>) {
        self.modules = Some(modules);
    }

    pub fn modules(&self) -> &Option<Vec<(&'m str, Module<'m>)>> {
        &self.modules
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
    /// Side-effects allowed anywhere on underlying system
    Unrestricted,

    /// Side-effects allowed within this document's folder only
    #[default]
    Standard,

    /// No side-effects on underlying system
    Strict,
}

#[derive(Copy, Clone, Debug, Default)]
pub enum ResourceLimit {
    Limited(usize),
    #[default]
    Unlimited,
}

impl ResourceLimit {
    pub fn as_usize(&self) -> usize {
        match self {
            Self::Limited(l) => *l,
            Self::Unlimited => usize::MAX,
        }
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
