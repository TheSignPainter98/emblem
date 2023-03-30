use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{self, Path, PathBuf}, process::{self, ExitCode}
};
// use thiserror::Error;
use mlua::{lua_State, Lua, Value};

extern "C" {
    fn luaopen_yue(state: *mut lua_State) -> std::os::raw::c_int;
}

pub struct Compiler {
    in_dir: PathBuf,
    out_dir: PathBuf,
    test: bool,
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

        for input_path in input_paths.iter() {
            let pretty_path = input_path.as_os_str().to_string_lossy();
            println!("cargo:rerun-if-changed={pretty_path}");
            println!("{}", std::env::current_dir()?.display());

            let module_name = input_path
                .strip_prefix(&self.in_dir)?
                .with_extension("")
                .as_os_str()
                .to_string_lossy()
                .replace(path::MAIN_SEPARATOR_STR, ".");

            let globals = lua.globals();
            globals.set("raw", fs::read_to_string(input_path)?)?;
            globals.set("test", self.test)?;
            globals.set("module_name", module_name.as_str())?;

            let bytecode: Vec<u8> = match lua
                .load(&compile_script)
                .set_name(format!("compile:{module_name}"))?
                .call(()) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("{e}");
                    process::exit(1);
                }
            };

            println!("Creating file {:?}", self.output_path(&input_path));
            let output_path = self.output_path(&input_path)?;
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = BufWriter::new(File::create(output_path)?);
            out_file.write_all(&bytecode)?;
        }

        Ok(())

        // input:
        //     find files
        //     figure out dependencies and hence load order
    }

    fn get_inputs(&self) -> Result<Vec<std::path::PathBuf>, Box<dyn Error>> {
        let mut to_compile = Vec::new();
        for entry in fs::read_dir(&self.in_dir)? {
            let entry = entry?;
            let path = entry.path();
            if entry.metadata()?.is_file()
                && path.extension().map(|e| e.to_string_lossy()) == Some("yue".into())
            {
                to_compile.push(path)
            }
        }
        Ok(to_compile)
    }

    fn output_path(&self, input_path: &Path) -> Result<PathBuf, Box<dyn Error>> {
        Ok(self
            .out_dir
            .join(input_path.strip_prefix(&self.in_dir)?)
            .with_extension("luac"))
    }
}
