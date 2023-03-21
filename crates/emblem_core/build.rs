use std::env;
use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    parsergen()?;
    // yuescriptgen()?;
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

// fn yuescriptgen() -> Result<(), Box<dyn Error>> {
//     println!("cargo:rerun-if-changed=build.rs");
//     cc::Build::new()
//         .cpp(true)
//         .include(".")
//         .include("deps/yuescript/src/")
//         .include(std::env::var("DEP_LUA_INCLUDE").unwrap())
//         .file("deps/yuescript/src/yuescript/ast.cpp")
//         .file("deps/yuescript/src/yuescript/parser.cpp")
//         .file("deps/yuescript/src/yuescript/yue_compiler.cpp")
//         .file("deps/yuescript/src/yuescript/yue_parser.cpp")
//         .file("deps/yuescript/src/yuescript/yuescript.cpp")
//         .flag_if_supported("-std=c++17")
//         .shared_flag(false)
//         .define("NDEBUG", None)
//         .define("YUE_NO_WATCHER", None)
//         .define("YUE_COMPILER_ONLY", None)
//         .compile("yue");

//     Ok(())
// }
