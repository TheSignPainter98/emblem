use std::env;
use std::error::Error;
use std::path::Path;

use yuescript::Compiler;

// extern "C" {
//     fn luaopen_yue(state: *mut lua_State) -> std::os::raw::c_int;
// }

fn main() -> Result<(), Box<dyn Error>> {
    parsergen()?;
    luagen()?;
    Ok(())
}

fn parsergen() -> Result<(), Box<dyn Error>> {
    let out_dir = Path::new(&env::var("OUT_DIR").unwrap()).join("parser");

    lalrpop::Configuration::new()
        .emit_rerun_directives(true)
        .set_in_dir("src/parser/")
        .set_out_dir(out_dir)
        .process()
}

fn luagen() -> Result<(), Box<dyn Error>> {
    let out_dir = Path::new(&env::var("OUT_DIR").unwrap())
        .join("yue")
        .join("std.luac");
    let compiler = Compiler::new("src/extensions/yuescript/", &out_dir).with_test(true); // TODO(kcza): figure out how to get tests to work!
    compiler.compile()?;

    Ok(())
}
