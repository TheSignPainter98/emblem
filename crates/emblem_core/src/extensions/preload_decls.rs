use crate::context::SandboxLevel;
use derive_new::new;
use mlua::{Function, Result as MLuaResult, Table, Value};

pub(crate) static PRELOADS: [PreloadedPackage; 11] = [
    PreloadedPackage {
        name: "jit.profile",
        sandbox_level: Some(SandboxLevel::Unrestricted),
        preload: false,
    },
    PreloadedPackage {
        name: "jit.util",
        sandbox_level: Some(SandboxLevel::Unrestricted),
        preload: false,
    },
    PreloadedPackage {
        name: "string.buffer",
        sandbox_level: Some(SandboxLevel::Strict),
        preload: false,
    },
    PreloadedPackage {
        name: "table.clear",
        sandbox_level: Some(SandboxLevel::Strict),
        preload: true,
    },
    PreloadedPackage {
        name: "table.clone",
        sandbox_level: Some(SandboxLevel::Strict),
        preload: true,
    },
    PreloadedPackage {
        name: "table.isarray",
        sandbox_level: Some(SandboxLevel::Strict),
        preload: true,
    },
    PreloadedPackage {
        name: "table.isempty",
        sandbox_level: Some(SandboxLevel::Strict),
        preload: true,
    },
    PreloadedPackage {
        name: "table.new",
        sandbox_level: Some(SandboxLevel::Strict),
        preload: true,
    },
    PreloadedPackage {
        name: "table.nkeys",
        sandbox_level: Some(SandboxLevel::Strict),
        preload: true,
    },
    PreloadedPackage {
        name: "thread.exdata",
        sandbox_level: None,
        preload: false,
    },
    PreloadedPackage {
        name: "thread.exdata2",
        sandbox_level: None,
        preload: false,
    },
];

#[derive(new)]
pub(crate) struct PreloadedPackage {
    name: &'static str,
    sandbox_level: Option<SandboxLevel>,
    preload: bool,
}

impl PreloadedPackage {
    /// Check whether this extension should be removed from Lua's cached functions.
    pub(crate) fn expunge_preloads(
        &self,
        package_preload: &Table,
        sandbox_level: SandboxLevel,
    ) -> MLuaResult<()> {
        if self.marked_for_expunge(sandbox_level) {
            package_preload.set(self.name, Value::Nil)?;
        }

        Ok(())
    }

    pub(crate) fn marked_for_expunge(&self, sandbox_level: SandboxLevel) -> bool {
        match self.sandbox_level {
            None => true,
            Some(l) => l < sandbox_level,
        }
    }

    pub(crate) fn handle_preload(&self, require: &Function) -> MLuaResult<()> {
        if !self.preload {
            return Ok(());
        }

        require.call(self.name)
    }
}

#[cfg(test)]
impl PreloadedPackage {
    pub(crate) fn name(&self) -> &str {
        self.name
    }

    pub(crate) fn marked_for_preload(&self) -> bool {
        self.preload
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mlua::Lua;
    use std::error::Error;

    #[test]
    fn all_preloads_constrained() -> Result<(), Box<dyn Error>> {
        let lua = unsafe { Lua::unsafe_new() };
        let package_preload = lua
            .globals()
            .get::<_, Table>("package")?
            .get::<_, Table>("preload")?;

        let mut unconstrained = Vec::new();
        for entry in package_preload.pairs() {
            let (mod_name, _): (String, Value) = entry?;
            if PRELOADS.iter().all(|p| p.name() != mod_name) {
                unconstrained.push(mod_name);
            }
        }

        assert!(
            unconstrained.is_empty(),
            "some preloads not constrained: {}",
            unconstrained.join(", ")
        );

        Ok(())
    }

    #[test]
    fn all_restricted_valid() -> Result<(), Box<dyn Error>> {
        let lua = unsafe { Lua::unsafe_new() };
        let package_preload = lua
            .globals()
            .get::<_, Table>("package")?
            .get::<_, Table>("preload")?;

        let mut unused = Vec::new();
        for preload in &PRELOADS {
            let name = preload.name();
            if package_preload.get::<_, Value>(name)? == Value::Nil {
                unused.push(name);
            }
        }

        assert!(
            unused.is_empty(),
            "some preload restrictions are never used: {}",
            unused.join(", ")
        );

        Ok(())
    }

    #[test]
    fn only_strict_preloadable() -> Result<(), Box<dyn Error>> {
        for preload in &PRELOADS {
            match (preload.sandbox_level, preload.preload) {
                (Some(SandboxLevel::Strict), _) => {}
                (l, true) => panic!("{} preloadable despite sandbox level {l:?}", preload.name()),
                _ => {}
            }
        }

        Ok(())
    }
}
