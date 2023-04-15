use crate::context::SandboxLevel;

pub(crate) static PRELOADS: [(&str, SandboxLevel, bool); 11] = [
    ("jit.profile", SandboxLevel::Unrestricted, false),
    ("jit.util", SandboxLevel::Unrestricted, false),
    ("string.buffer", SandboxLevel::Strict, false),
    ("table.clear", SandboxLevel::Strict, true),
    ("table.clone", SandboxLevel::Strict, true),
    ("table.isarray", SandboxLevel::Strict, true),
    ("table.isempty", SandboxLevel::Strict, true),
    ("table.new", SandboxLevel::Strict, true),
    ("table.nkeys", SandboxLevel::Strict, true),
    ("thread.exdata", SandboxLevel::Unsound, false),
    ("thread.exdata2", SandboxLevel::Unsound, false),
];

#[cfg(test)]
mod test {
    use super::*;
    use mlua::{Lua, Table, Value};
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
            if PRELOADS.iter().all(|(k, _, _)| *k != mod_name) {
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
        for (k, _, _) in PRELOADS {
            if package_preload.get::<_, Value>(k)? == Value::Nil {
                unused.push(k);
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
        for entry in PRELOADS {
            match entry {
                (_, SandboxLevel::Strict, _) => {}
                (k, l, true) => panic!("{k} preloadable despite sandbox level {l:?}"),
                _ => {}
            }
        }

        Ok(())
    }
}
