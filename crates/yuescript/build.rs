use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    yuescriptgen()?;
    Ok(())
}

fn yuescriptgen() -> Result<(), Box<dyn Error>> {
    cc::Build::new()
        .cpp(true)
        .include(".")
        .include("yuescript/src/")
        .include(env::var("DEP_LUA_INCLUDE").unwrap())
        .file("yuescript/src/yuescript/ast.cpp")
        .file("yuescript/src/yuescript/parser.cpp")
        .file("yuescript/src/yuescript/yue_compiler.cpp")
        .file("yuescript/src/yuescript/yue_parser.cpp")
        .file("yuescript/src/yuescript/yuescript.cpp")
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-Wno-implicit-fallthrough")
        .static_flag(true)
        .define("NDEBUG", None)
        .define("YUE_NO_WATCHER", None)
        .define("YUE_COMPILER_ONLY", None)
        .compile("yue");

    Ok(())
}
