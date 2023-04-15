use mlua::{Lua, Result as MLuaResult, Function};
use crate::extensions::preload_decls;

pub(crate) fn import_extras(lua: &Lua) -> MLuaResult<()> {
    let require: Function = lua.globals().get("require")?;

    for module in preload_decls::PRELOADS
        .iter()
        .filter_map(|(m, _, p)| if *p { Some(m) } else { None })
    {
        require.call(*module)?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{collections::HashSet, error::Error};
    use mlua::chunk;

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
            for (module_name, _, preload) in preload_decls::PRELOADS {
                if !preload {
                    continue
                }
                assert!(test_set.contains(&module_name), "{module_name} not tested!");
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
