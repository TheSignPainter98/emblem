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
    let out_dir = Path::new(&env::var("OUT_DIR").unwrap()).join("yue");
    let compiler = Compiler::new("asdf", &out_dir);
    compiler.compile()?;
    // let lua = Lua::new();
    // let yue = unsafe { lua.create_c_function(luaopen_yue)? };
    // lua.load_from_function::<_, Value>("yue", yue)?;
    // lua.load(
    //     chunk! {
    //         local yue = require("yue")
    //             local codes, err, globals = yue.to_lua([[print "hello, world"]], {
    //                 implicit_return_root = true,
    //                 reserve_line_number = true,
    //                 lint_global = true,
    //             })
    //         if err then
    //             error(err)
    //                 end
    //                 load(codes)()
    //     }
    //     ).exec()?;

    Ok(())
}
