use crate::context::SandboxLevel;
use mlua::{Lua, Result as MLuaResult, Table, Value};

pub(crate) fn restrict_preload(lua: &Lua, sandbox_level: SandboxLevel) -> MLuaResult<()> {
    let package_preload = lua
        .globals()
        .get::<_, Table>("package")?
        .get::<_, Table>("preload")?;

    for (k, l) in RESTRICTIONS {
        if l < sandbox_level {
            package_preload.set(k, Value::Nil)?;
        }
    }

    Ok(())
}

const RESTRICTIONS: [(&str, SandboxLevel); 11] = [
    ("jit.profile", SandboxLevel::Unrestricted),
    ("jit.util", SandboxLevel::Unrestricted),
    ("string.buffer", SandboxLevel::Strict),
    ("table.clear", SandboxLevel::Strict),
    ("table.clone", SandboxLevel::Strict),
    ("table.isarray", SandboxLevel::Strict),
    ("table.isempty", SandboxLevel::Strict),
    ("table.new", SandboxLevel::Strict),
    ("table.nkeys", SandboxLevel::Strict),
    ("thread.exdata", SandboxLevel::Unsound),
    ("thread.exdata2", SandboxLevel::Unsound),
];

#[cfg(test)]
mod test {
    use super::*;
    use mlua::{chunk, Table, Value};
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
            if RESTRICTIONS.iter().all(|(k, _)| *k != mod_name) {
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
        for (k, _) in RESTRICTIONS {
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
    fn preloads_unloaded() -> Result<(), Box<dyn Error>> {
        for sandbox_level in SandboxLevel::input_levels() {
            let lua = unsafe { Lua::unsafe_new() };
            restrict_preload(&lua, sandbox_level)?;

            for (k, l) in RESTRICTIONS {
                let expect_removed = l < sandbox_level;
                lua.load(chunk! {
                    local ok, err = pcall(require, $k);
                    if $expect_removed then
                        assert(not ok, "loading " .. $k .. " succeeded");
                        assert(err:match("^module '" .. $k .. "' not found:"), "unexpected error: " .. err);
                    else
                        assert(ok, "unexpected error loading module $k: " .. tostring(err));
                    end;
                })
                .exec()
                .map_err(|e| panic!("{e}"))
                .unwrap();
            }
        }

        Ok(())
    }
}
