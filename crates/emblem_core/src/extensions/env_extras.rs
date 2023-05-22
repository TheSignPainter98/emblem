use crate::extensions::preload_decls;
use mlua::{Function, Lua, Result as MLuaResult};

pub(crate) fn import_extras(lua: &Lua) -> MLuaResult<()> {
    let require: Function = lua.globals().get("require")?;
    for preload in &preload_decls::PRELOADS {
        preload.handle_preload(&require)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use mlua::chunk;
    use std::{collections::HashSet, error::Error};

    #[test]
    fn extra_imports() -> Result<(), Box<dyn Error>> {
        let lua = Lua::new();

        let tests = [
            ("table.clear", "table.clear", "function"),
            ("table.clone", "table.clone", "function"),
            ("table.isarray", "table.isarray", "function"),
            ("table.isempty", "table.isempty", "function"),
            ("table.new", "table.new", "function"),
            ("table.nkeys", "table.nkeys", "function"),
        ];

        {
            let test_set = tests.iter().map(|(m, _, _)| m).collect::<HashSet<&&str>>();
            for preload in &preload_decls::PRELOADS {
                if !preload.marked_for_preload() {
                    continue;
                }
                let name = preload.name();
                assert!(test_set.contains(&name), "{name} not tested!");
            }
        }

        for (module, check_path, expected_type) in tests {
            lua.load(chunk! {
                require($module);
                local t = type(assert(loadstring("return " .. $check_path))());
                assert(t == $expected_type, $check_path .. " has incorrect type: " .. t);
            })
            .exec()?;
        }

        Ok(())
    }
}
