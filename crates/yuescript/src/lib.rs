use mlua::{lua_State, Lua, Value};
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{self, Path, PathBuf},
    process,
};
// use thiserror::Error;

#[macro_export]
macro_rules! include_yuescript {
    ($test:expr, $dir:expr, $name:literal) => {
        if !$test {
            include_bytes!(concat!($dir, "/", $name, ".luac"))
        } else {
            include_bytes!(concat!($dir, "/", $name, ".spec.luac"))
        }
    };
}

extern "C" {
    fn luaopen_yue(state: *mut lua_State) -> std::os::raw::c_int;
    fn luaopen_lfs(state: *mut lua_State) -> std::os::raw::c_int;
    fn luaopen_term_core(state: *mut lua_State) -> std::os::raw::c_int;
    fn luaopen_system_core(state: *mut lua_State) -> std::os::raw::c_int;
}

pub struct Compiler {
    in_dir_path: PathBuf,
    out_path: PathBuf,
    name: String,
    compile_script: String,
}

impl Compiler {
    pub fn new<S, T, U>(in_dir: S, out_path: T, name: U) -> Result<Self, Box<dyn Error>>
    where
        S: Into<PathBuf>,
        T: Into<PathBuf>,
        U: Into<String>,
    {
        let compile_script_path = env::current_dir()?
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(file!())
            .with_file_name("compile.lua");
        println!("cargo:rerun-if-changed={}", compile_script_path.display());
        let compile_script = fs::read_to_string(compile_script_path)?;
        Ok(Self {
            in_dir_path: in_dir.into(),
            out_path: out_path.into(),
            name: name.into(),
            compile_script,
        })
    }

    pub fn compile(&self) -> Result<(), Box<dyn Error>> {
        let input_paths = self.get_inputs()?;
        let out_file = self.out_path.join(&self.name).with_extension("luac");

        self.compile_unit(&input_paths, &out_file, false)?;
        self.compile_unit(&input_paths, out_file.with_extension("spec.luac"), true)?;

        Ok(())
    }

    fn compile_unit(
        &self,
        input_paths: &[PathBuf],
        out_file: impl AsRef<Path>,
        generate_test: bool,
    ) -> Result<(), Box<dyn Error>> {
        let out_file = out_file.as_ref();

        let lua = Lua::new();
        let yue = unsafe { lua.create_c_function(luaopen_yue)? };
        lua.load_from_function::<_, Value>("yue", yue)?;

        let globals = lua.globals();
        globals.set("test", generate_test)?;
        globals.set("dep_dir", self.dep_dir()?)?;
        globals.set("inputs", {
            let tab = lua.create_table_with_capacity(0, input_paths.len() as i32)?;
            for input_path in input_paths {
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

        let bytecode: Vec<u8> = match lua.load(&self.compile_script).set_name("compile")?.call(()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
        };

        if let Some(parent) = out_file.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = BufWriter::new(File::create(out_file)?);
        out.write_all(&bytecode)?;

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
        let mut dep_dir: String = current_file
            .ancestors()
            .nth(2)
            .unwrap()
            .to_string_lossy()
            .into();
        if path::MAIN_SEPARATOR == '\\' {
            dep_dir = dep_dir.replace(path::MAIN_SEPARATOR, "/"); // >:C
        }
        Ok(dep_dir)
    }
}

pub struct Tester {}

impl Tester {
    pub fn new() -> Self {
        Self {}
    }

    pub fn test(&self, lua: &mlua::Lua) {
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

        lua.load(
            r#"
                local __luatest, err = require('__luatest')
                if err ~= nil then
                    error('failed to load tests:\n' .. err)
                end
                __luatest()
            "#,
        )
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
