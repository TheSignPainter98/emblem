use crate::{context::SandboxLevel, extensions::preload_decls, Result};
use mlua::{Lua, Table};

pub(crate) fn restrict_preload(lua: &Lua, sandbox_level: SandboxLevel) -> Result<()> {
    let package_preload = lua
        .globals()
        .get::<_, Table>("package")?
        .get::<_, Table>("preload")?;
    for preload in &preload_decls::PRELOADS {
        preload.expunge_preloads(&package_preload, sandbox_level)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use mlua::chunk;

    #[test]
    fn preloads_expunged() -> Result<()> {
        for sandbox_level in SandboxLevel::input_levels() {
            let lua = unsafe { Lua::unsafe_new() };
            restrict_preload(&lua, sandbox_level)?;

            for preload in &preload_decls::PRELOADS {
                let name = preload.name();
                if preload.marked_for_expunge(sandbox_level) {
                    lua.load(chunk! {
                        local ok, err = pcall(require, $name);
                        assert(not ok, "loading " .. $name .. " succeeded");
                        assert(err:match("^module '" .. $name .. "' not found:"), "unexpected error: " .. err);
                    })
                    .exec()
                    .map_err(|e| panic!("{e}"))
                    .unwrap();
                } else {
                    lua.load(chunk! {
                        local ok, err = pcall(require, $name);
                        assert(ok, "unexpected error loading module $k: " .. tostring(err));
                    })
                    .exec()
                    .map_err(|e| panic!("{e}"))
                    .unwrap();
                }
            }
        }

        Ok(())
    }
}
