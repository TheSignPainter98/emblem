use crate::{extensions::preload_decls, context::SandboxLevel};
use mlua::{Lua, Result as MLuaResult, Table, Value};

pub(crate) fn restrict_preload(lua: &Lua, sandbox_level: SandboxLevel) -> MLuaResult<()> {
    let package_preload = lua
        .globals()
        .get::<_, Table>("package")?
        .get::<_, Table>("preload")?;

    for (k, l, _) in preload_decls::PRELOADS {
        if l < sandbox_level {
            package_preload.set(k, Value::Nil)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use mlua::chunk;
    use std::error::Error;

    #[test]
    fn preloads_unloaded() -> Result<(), Box<dyn Error>> {
        for sandbox_level in SandboxLevel::input_levels() {
            let lua = unsafe { Lua::unsafe_new() };
            restrict_preload(&lua, sandbox_level)?;

            for (k, l, _) in preload_decls::PRELOADS {
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
