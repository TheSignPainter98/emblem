pub(crate) mod file_content;
pub(crate) mod file_name;
mod module;
mod resource_limit;
mod resources;

use crate::{ExtensionState, FileContent, FileName, Typesetter, Version};
use derive_new::new;
use mlua::Result as MLuaResult;
pub use module::{Module, ModuleVersion};
use once_cell::unsync::OnceCell;
pub use resource_limit::ResourceLimit;
pub use resources::{Iteration, Memory, Resource, Step};

#[derive(Default)]
pub struct Context {
    doc_params: DocumentParameters,
    lua_params: LuaParameters,
    typesetter_params: TypesetterParameters,
    extension_state: OnceCell<ExtensionState>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc_file_name(&self, name: impl AsRef<str>) -> FileName {
        FileName::new(name.as_ref())
    }

    pub fn alloc_file_content(&self, content: impl AsRef<str>) -> FileContent {
        FileContent::new(content.as_ref())
    }

    pub fn doc_params(&self) -> &DocumentParameters {
        &self.doc_params
    }

    pub fn doc_params_mut(&mut self) -> &mut DocumentParameters {
        &mut self.doc_params
    }

    pub fn lua_params(&self) -> &LuaParameters {
        &self.lua_params
    }

    pub fn lua_params_mut(&mut self) -> &mut LuaParameters {
        &mut self.lua_params
    }

    pub fn typesetter_params(&self) -> &TypesetterParameters {
        &self.typesetter_params
    }

    pub fn typesetter_params_mut(&mut self) -> &mut TypesetterParameters {
        &mut self.typesetter_params
    }

    pub fn extension_state(&self) -> MLuaResult<&ExtensionState> {
        self.extension_state
            .get_or_try_init(|| ExtensionState::new(self))
    }

    pub fn typesetter(&self) -> Typesetter {
        Typesetter::new(self)
    }
}

#[cfg(test)]
impl Context {
    pub fn test_new() -> Self {
        Self {
            doc_params: DocumentParameters::test_new(),
            lua_params: LuaParameters::test_new(),
            typesetter_params: TypesetterParameters::test_new(),
            extension_state: OnceCell::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct DocumentParameters {
    name: Option<String>,
    emblem_version: Option<Version>,
    authors: Option<Vec<String>>,
    keywords: Option<Vec<String>>,
}

impl DocumentParameters {
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn set_emblem_version(&mut self, emblem_version: Version) {
        self.emblem_version = Some(emblem_version);
    }

    pub fn emblem_version(&self) -> &Option<Version> {
        &self.emblem_version
    }

    pub fn set_authors(&mut self, authors: Vec<String>) {
        self.authors = Some(authors);
    }

    pub fn authors(&self) -> Option<&[String]> {
        self.authors.as_deref()
    }

    pub fn set_keywords(&mut self, keywords: Vec<String>) {
        self.keywords = Some(keywords);
    }

    pub fn keywords(&self) -> Option<&[String]> {
        self.keywords.as_deref()
    }
}

#[cfg(test)]
impl DocumentParameters {
    pub fn test_new() -> Self {
        Self {
            name: Some("On the Origin of Burnt Toast".into()),
            emblem_version: Some(Version::V1_0),
            authors: Some(vec!["kcza".into()]),
            keywords: Some(
                ["toast", "burnt", "backstory"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            ),
        }
    }
}

#[derive(new, Debug, Default)]
pub struct LuaParameters {
    sandbox_level: SandboxLevel,
    max_mem: ResourceLimit<Memory>,
    max_steps: ResourceLimit<Step>,
    general_args: Option<Vec<(String, String)>>,
    modules: Vec<Module>,
}

impl LuaParameters {
    pub fn set_sandbox_level(&mut self, sandbox_level: SandboxLevel) {
        self.sandbox_level = sandbox_level;
    }

    pub fn sandbox_level(&self) -> SandboxLevel {
        self.sandbox_level
    }

    pub fn set_max_mem(&mut self, max_mem: ResourceLimit<Memory>) {
        self.max_mem = max_mem;
    }

    pub fn max_mem(&self) -> ResourceLimit<Memory> {
        self.max_mem
    }

    pub fn set_max_steps(&mut self, max_steps: ResourceLimit<Step>) {
        self.max_steps = max_steps;
    }

    pub fn max_steps(&self) -> ResourceLimit<Step> {
        self.max_steps
    }

    pub fn set_general_args(&mut self, general_args: Vec<(String, String)>) {
        self.general_args = Some(general_args);
    }

    pub fn general_args(&self) -> Option<&[(String, String)]> {
        self.general_args.as_deref()
    }

    pub fn set_modules(&mut self, modules: Vec<Module>) {
        self.modules = modules;
    }

    pub fn modules(&self) -> &[Module] {
        &self.modules
    }
}

#[cfg(test)]
impl LuaParameters {
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

#[derive(Debug, Default)]
pub struct TypesetterParameters {
    max_iters: ResourceLimit<Iteration>,
}

impl TypesetterParameters {
    pub fn max_iters(&self) -> ResourceLimit<Iteration> {
        self.max_iters
    }

    pub fn set_max_iters(&mut self, max_iters: ResourceLimit<Iteration>) {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn alloc_file_name() {
        let ctx = Context::test_new();
        let name = "/usr/share/man/man1/gcc.1.gz";

        let result = ctx.alloc_file_name(name);
        assert_eq!(result, name);
    }

    #[test]
    fn alloc_file() {
        let ctx = Context::test_new();
        let content = "hello, world".to_owned();

        let result = ctx.alloc_file_content(content.clone());
        assert_eq!(result.as_ref(), content);
    }
}
