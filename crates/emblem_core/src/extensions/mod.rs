use mlua::Lua;

pub fn test() {
    let lua = Lua::new();

    lua.load(
        r#"
            print("hello, world from Lua!")
        "#,
    )
    .eval::<()>()
    .unwrap();

    for global in lua.globals().pairs::<String, mlua::Value>() {
        println!("{:?}", global);
    }
}

#[derive(Debug, Default)]
enum SandboxLevel {
    /// Modules have no restrictions placed upon them.
    Unrestricted,

    /// Prohibit creation of new subprocesses and file system access outside of the current
    /// working directory.
    #[default]
    Standard,

    /// Same restrictions as Standard, but all file system access if prohibited.
    Strict,
}

// TODO(kcza): go through the globals and set the sandbox level of each!
