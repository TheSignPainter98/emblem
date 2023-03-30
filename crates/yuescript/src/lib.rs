use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{self, Path, PathBuf},
    process,
};
// use thiserror::Error;
use mlua::{lua_State, Lua, Value};

extern "C" {
    fn luaopen_yue(state: *mut lua_State) -> std::os::raw::c_int;
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
        globals.set("luacheck_path", self.luacheck_path()?)?;
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

    fn luacheck_path(&self) -> Result<String, Box<dyn Error>> {
        let current_file = PathBuf::from(env::current_dir()?)
            .ancestors()
            .nth(2)
            .unwrap()
            .join(file!());
        let crate_dir = current_file.ancestors().nth(2).unwrap();
        Ok(format!(
            "{}/luacheck/src/?.lua;{}/luacheck/src/?/init.lua",
            crate_dir.display(),
            crate_dir.to_string_lossy().replace("luacheck", "?")
        ))
    }
}
