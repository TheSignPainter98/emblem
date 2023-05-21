mod module;

use crate::{ExtensionState, Typesetter, Version};
use derive_new::new;
use mlua::Result as MLuaResult;
pub use module::{Module, ModuleVersion};
use num::{Bounded, Integer};
use std::fmt::Debug;
use typed_arena::Arena;

pub const DEFAULT_MAX_STEPS: u32 = 100_000;
pub const DEFAULT_MAX_MEM: usize = 100_000;
pub const DEFAULT_MAX_ITERS: u32 = 5;

#[derive(Default)]
pub struct Context<'m> {
    files: Arena<File>,
    doc_params: DocumentParameters<'m>,
    lua_params: LuaParameters<'m>,
    typesetter_params: TypesetterParameters,
}

impl<'m> Context<'m> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc_file(&self, name: String, content: String) -> &File {
        self.files.alloc(File { name, content })
    }

    pub fn doc_params(&self) -> &DocumentParameters<'m> {
        &self.doc_params
    }

    pub fn doc_params_mut(&mut self) -> &mut DocumentParameters<'m> {
        &mut self.doc_params
    }

    pub fn lua_params(&self) -> &LuaParameters<'m> {
        &self.lua_params
    }

    pub fn lua_params_mut(&mut self) -> &mut LuaParameters<'m> {
        &mut self.lua_params
    }

    pub fn typesetter_params(&self) -> &TypesetterParameters {
        &self.typesetter_params
    }

    pub fn typesetter_params_mut(&mut self) -> &mut TypesetterParameters {
        &mut self.typesetter_params
    }

    pub fn extension_state(&'m self) -> MLuaResult<ExtensionState<'m>> {
        ExtensionState::new(self)
    }

    pub fn typesetter(&'m self, ext_state: &'m mut ExtensionState<'m>) -> Typesetter<'m> {
        Typesetter::new(self, ext_state)
    }
}

#[cfg(test)]
impl<'m> Context<'m> {
    pub fn test_new() -> Self {
        Self {
            files: Arena::new(),
            doc_params: DocumentParameters::test_new(),
            lua_params: LuaParameters::test_new(),
            typesetter_params: TypesetterParameters::test_new(),
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
pub struct DocumentParameters<'m> {
    name: Option<&'m str>,
    emblem_version: Option<Version>,
    authors: Option<Vec<&'m str>>,
    keywords: Option<Vec<&'m str>>,
}

impl<'m> DocumentParameters<'m> {
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

#[cfg(test)]
impl<'m> DocumentParameters<'m> {
    pub fn test_new() -> Self {
        Self {
            name: Some("On the Origin of Burnt Toast"),
            emblem_version: Some(Version::V1_0),
            authors: Some(vec!["kcza"]),
            keywords: Some(vec!["toast", "burnt", "backstory"]),
        }
    }
}

#[derive(new, Debug)]
pub struct LuaParameters<'m> {
    sandbox_level: SandboxLevel,
    max_mem: ResourceLimit<usize>,
    max_steps: ResourceLimit<u32>,
    general_args: Option<Vec<(&'m str, &'m str)>>,
    modules: Vec<Module<'m>>,
}

impl<'m> Default for LuaParameters<'m> {
    fn default() -> Self {
        Self {
            sandbox_level: Default::default(),
            max_mem: ResourceLimit::Limited(DEFAULT_MAX_MEM),
            max_steps: ResourceLimit::Limited(DEFAULT_MAX_STEPS),
            general_args: Default::default(),
            modules: Default::default(),
        }
    }
}

impl<'m> LuaParameters<'m> {
    pub fn set_sandbox_level(&mut self, sandbox_level: SandboxLevel) {
        self.sandbox_level = sandbox_level;
    }

    pub fn sandbox_level(&self) -> SandboxLevel {
        self.sandbox_level
    }

    pub fn set_max_mem(&mut self, max_mem: ResourceLimit<usize>) {
        self.max_mem = max_mem;
    }

    pub fn max_mem(&self) -> ResourceLimit<usize> {
        self.max_mem
    }

    pub fn set_max_steps(&mut self, max_steps: ResourceLimit<u32>) {
        self.max_steps = max_steps;
    }

    pub fn max_steps(&self) -> ResourceLimit<u32> {
        self.max_steps
    }

    pub fn set_general_args(&mut self, general_args: Vec<(&'m str, &'m str)>) {
        self.general_args = Some(general_args);
    }

    pub fn general_args(&self) -> &Option<Vec<(&'m str, &'m str)>> {
        &self.general_args
    }

    pub fn set_modules(&mut self, modules: Vec<Module<'m>>) {
        self.modules = modules;
    }

    pub fn modules(&self) -> &[Module<'m>] {
        &self.modules
    }
}

#[cfg(test)]
impl<'m> LuaParameters<'m> {
    pub fn test_new() -> Self {
        Self {
            sandbox_level: SandboxLevel::Strict,
            max_mem: ResourceLimit::Unlimited,
            max_steps: ResourceLimit::Unlimited,
            general_args: None,
            modules: vec![],
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum SandboxLevel {
    /// Can break Emblem's abstractions
    Unsound,

    /// Side-effects allowed anywhere on host system
    Unrestricted,

    /// Side-effects allowed within this document's folder only
    #[default]
    Standard,

    /// No side-effects on host system
    Strict,
}

#[cfg(test)]
impl SandboxLevel {
    pub fn input_levels() -> impl Iterator<Item = SandboxLevel> {
        [
            SandboxLevel::Unrestricted,
            SandboxLevel::Standard,
            SandboxLevel::Strict,
        ]
        .into_iter()
    }
}

pub struct TypesetterParameters {
    max_iters: ResourceLimit<u32>,
}

impl Default for TypesetterParameters {
    fn default() -> Self {
        Self {
            max_iters: ResourceLimit::Limited(DEFAULT_MAX_ITERS),
        }
    }
}

impl TypesetterParameters {
    pub fn max_iters(&self) -> ResourceLimit<u32> {
        self.max_iters
    }

    pub fn set_max_iters(&mut self, max_iters: ResourceLimit<u32>) {
        self.max_iters = max_iters
    }
}

#[cfg(test)]
impl TypesetterParameters {
    pub fn test_new() -> Self {
        Self {
            max_iters: ResourceLimit::Unlimited,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ResourceLimit<T: Bounded + Clone + Integer> {
    Unlimited,
    Limited(T),
}

impl<T: Bounded + Clone + Integer> ResourceLimit<T> {
    pub(crate) fn limit(&self) -> Option<T> {
        match self {
            Self::Unlimited => None,
            Self::Limited(l) => Some(l.clone()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn alloc_file() {
        let ctx = Context::test_new();
        let name = "/usr/share/man/man1/gcc.1.gz".to_owned();
        let content = "hello, world".to_owned();

        let file = ctx.alloc_file(name.clone(), content.clone());
        assert_eq!(file.name(), name);
        assert_eq!(file.content(), content);
    }
}
