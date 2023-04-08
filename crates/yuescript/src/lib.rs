use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{self, PathBuf},
    process,
};
// use thiserror::Error;
use mlua::{lua_State, Lua, Value};

extern "C" {
    fn luaopen_yue(state: *mut lua_State) -> std::os::raw::c_int;
    fn luaopen_lfs(state: *mut lua_State) -> std::os::raw::c_int;
    fn luaopen_term_core(state: *mut lua_State) -> std::os::raw::c_int;
    fn luaopen_system_core(state: *mut lua_State) -> std::os::raw::c_int;
}

pub struct Compiler {
    in_dir_path: PathBuf,
    out_path: PathBuf,
    test: bool,
}

impl Compiler {
    pub fn new<S, T>(in_dir: S, out_path: T) -> Self
    where
        S: Into<PathBuf>,
        T: Into<PathBuf>,
    {
        Self {
            in_dir_path: in_dir.into(),
            out_path: out_path.into(),
            test: false,
        }
    }

    pub fn with_test(mut self, test: bool) -> Self {
        self.test = test;
        self
    }

    pub fn compile(&self) -> Result<(), Box<dyn Error>> {
        let compile_script_path = env::current_dir()?
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(file!())
            .with_file_name("compile.lua");
        println!("cargo:rerun-if-changed=src/compile.lua");
        let compile_script = fs::read_to_string(compile_script_path)?;

        let input_paths = self.get_inputs()?;

        let lua = Lua::new();
        let yue = unsafe { lua.create_c_function(luaopen_yue)? };
        lua.load_from_function::<_, Value>("yue", yue)?;

        let globals = lua.globals();
        globals.set("test", self.test)?;
        globals.set("dep_dir", self.dep_dir()?)?;
        globals.set("inputs", {
            let tab = lua.create_table_with_capacity(0, input_paths.len() as i32)?;
            for input_path in &input_paths {
                let module_name = input_path
                    .strip_prefix(&self.in_dir_path)?
                    .with_extension("")
                    .as_os_str()
                    .to_string_lossy()
                    .replace(path::MAIN_SEPARATOR, ".");
                tab.set(module_name, fs::read_to_string(input_path)?)?;
            }
            tab
        })?;

        let bytecode: Vec<u8> = match lua.load(&compile_script).set_name("compile")?.call(()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
        };

        if let Some(parent) = self.out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out_file = BufWriter::new(File::create(&self.out_path)?);
        out_file.write_all(&bytecode)?;

        Ok(())
    }

    fn get_inputs(&self) -> Result<Vec<std::path::PathBuf>, Box<dyn Error>> {
        let mut to_compile = Vec::new();
        for entry in fs::read_dir(&self.in_dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if entry.metadata()?.is_file()
                && path.extension().map(|e| e.to_string_lossy()) == Some("yue".into())
            {
                println!("cargo:rerun-if-changed={}", path.display());
                to_compile.push(path)
            }
        }
        Ok(to_compile)
    }

    fn dep_dir(&self) -> Result<String, Box<dyn Error>> {
        let current_file = env::current_dir()?
            .ancestors()
            .nth(2)
            .unwrap()
            .join(file!());
        Ok(current_file
            .ancestors()
            .nth(2)
            .unwrap()
            .to_string_lossy()
            .into())
    }
}

pub struct Tester {}

impl Tester {
    pub fn new() -> Self {
        Self {}
    }

    pub fn test(&self, lua: mlua::Lua) {
        lua.load_from_function::<_, Value>("lfs", unsafe {
            lua.create_c_function(luaopen_lfs).unwrap()
        })
        .unwrap();
        lua.load_from_function::<_, Value>("system", unsafe {
            lua.create_c_function(luaopen_system_core).unwrap()
        })
        .unwrap();
        lua.load_from_function::<_, Value>("term.core", unsafe {
            lua.create_c_function(luaopen_term_core).unwrap()
        })
        .unwrap();

        lua.load("require('emtest')()")
            .exec()
            .map_err(|e| {
                eprintln!("{e}");
                "error running tests"
            })
            .unwrap()
    }
}

impl Default for Tester {
    fn default() -> Self {
        Self::new()
    }
}
