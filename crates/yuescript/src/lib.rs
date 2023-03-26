use std::{error::Error, fs, path::PathBuf};
// use thiserror::Error;
use mlua::{chunk, lua_State, Lua, Value};

extern "C" {
    fn luaopen_yue(state: *mut lua_State) -> std::os::raw::c_int;
}

pub struct Compiler {
    in_dir: PathBuf,
    out_dir: PathBuf,
}

impl Compiler {
    pub fn new<S, T>(in_dir: S, out_dir: T) -> Self
    where
        S: Into<PathBuf>,
        T: Into<PathBuf>,
    {
        Self {
            in_dir: in_dir.into(),
            out_dir: out_dir.into(),
        }
    }

    pub fn compile(&self) -> Result<(), Box<dyn Error>> {
        let to_compile = {
            let mut to_compile = Vec::new();
            for entry in fs::read_dir(&self.in_dir)? {
                let entry = entry?;
                let path = entry.path();
                if entry.metadata()?.is_file()
                    && path.extension().map(|e| e.to_string_lossy()) == Some("yue".into())
                {
                    to_compile.push(entry)
                }
            }
            to_compile
        };

        let lua = Lua::new();
        let yue = unsafe { lua.create_c_function(luaopen_yue)? };
        lua.load_from_function::<_, Value>("yue", yue)?;
        lua.load(chunk! {
            local yue = require("yue")
            local codes, err, globals = yue.to_lua([[print "hello, world"]], {
                implicit_return_root = true,
                reserve_line_number = true,
                lint_global = true,
            })
            if err then
                error(err)
                    end
                    load(codes)()
        })
        .exec()?;

        Ok(())

        // input:
        //     find files
        //     figure out dependencies and hence load order
        // compilation:
        //     typecheck with teal?
        //     compile to bytecode
    }
}
