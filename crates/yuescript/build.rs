use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    yuescriptgen()?;
    luatermgen()?;
    luasystemgen()?;
    lfsgen()?;
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
        .flag_if_supported("-Wall")
        .flag_if_supported("-Wextra")
        .flag_if_supported("-Wno-deprecated-declarations")
        .flag_if_supported("-Wno-implicit-fallthrough")
        .static_flag(true)
        .define("NDEBUG", None)
        .define("YUE_NO_WATCHER", None)
        .define("YUE_COMPILER_ONLY", None)
        .compile("yue");

    Ok(())
}

fn lfsgen() -> Result<(), Box<dyn Error>> {
    cc::Build::new()
        .include("luafilesystem/src/")
        .include(env::var("DEP_LUA_INCLUDE").unwrap())
        .file("luafilesystem/src/lfs.c")
        .flag_if_supported("-Wall")
        .flag_if_supported("-Wextra")
        .compile("lfs");

    Ok(())
}

fn luatermgen() -> Result<(), Box<dyn Error>> {
    cc::Build::new()
        .include("lua-term/")
        .include(env::var("DEP_LUA_INCLUDE").unwrap())
        .file("lua-term/core.c")
        .flag_if_supported("-Wall")
        .flag_if_supported("-pedantic")
        .compile("lua-term");

    Ok(())
}

fn luasystemgen() -> Result<(), Box<dyn Error>> {
    cc::Build::new()
        .include("luasystem/src/")
        .include(env::var("DEP_LUA_INCLUDE").unwrap())
        // .file("luasystem/src/compat.c")
        .file("luasystem/src/core.c")
        .file("luasystem/src/time.c")
        .compile("luasystem");

    Ok(())
}
