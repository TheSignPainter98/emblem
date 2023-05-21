use crate::SandboxLevel;
use mlua::{Error as MLuaError, Lua, Result as MLuaResult, Table, Value};
use phf::{phf_map, Map};

pub(crate) fn restrict_globals(lua: &Lua, level: SandboxLevel) -> MLuaResult<()> {
    restrict_table(lua, level, lua.globals(), &CONSTRAINTS)
}

fn restrict_table(
    lua: &Lua,
    level: SandboxLevel,
    table: Table,
    constraints: &Map<&'static str, Constraint>,
) -> MLuaResult<()> {
    let mut to_replace = Vec::new();
    for entry in table.clone().pairs() {
        let (k, v): (String, Value) = entry?;

        match constraints.get(&k).unwrap_or_else(|| {
            panic!("internal error: unknown key {k:?} encountered while table fields")
        }) {
            Constraint::AtMost(l, rep) => {
                if *l < level {
                    let replacement = match rep {
                        None => Value::Nil,
                        Some(r) => r.make(lua, level)?,
                    };
                    to_replace.push((k, replacement));
                }
            }
            Constraint::Table(child_levels) => {
                if let Value::Table(t) = v {
                    restrict_table(lua, level, t, child_levels)?;
                } else {
                    panic!("internal error: expected table in {k}");
                }
            }
        }
    }

    for (k, v) in to_replace.clone() {
        table.set(k, v)?;
    }

    Ok(())
}

enum Constraint {
    AtMost(SandboxLevel, Option<Replacement>),
    Table(Map<&'static str, Constraint>),
}

enum Replacement {
    Nil,
    NilFunc,
    ErrFunc(&'static str),
    // Custom(&'static dyn for<'lua> Fn(&'lua Lua, SandboxLevel) -> MLuaResult<Value<'lua>>),
}

impl Replacement {
    fn make<'lua>(&self, lua: &'lua Lua, _level: SandboxLevel) -> MLuaResult<Value<'lua>> {
        Ok(match self {
            Self::Nil => Value::Nil,
            Self::NilFunc => Value::Function(lua.create_function(|_, ()| Ok(Value::Nil))?),
            Self::ErrFunc(name) => {
                let name: &'static str = name;
                Value::Function(lua.create_function(move |_, ()| -> MLuaResult<Value> {
                    Err(MLuaError::SafetyError(format!(
                        "function {name} unavailable to the sandbox"
                    )))
                })?)
            } // Self::Custom(f) => f(lua, level)?,
        })
    }
}

const CONSTRAINTS: Map<&'static str, Constraint> = phf_map! {
    // Values
    "_G"       => Constraint::AtMost(SandboxLevel::Strict, None),
    "_VERSION" => Constraint::AtMost(SandboxLevel::Strict, None),

    // Functions
    "assert"         => Constraint::AtMost(SandboxLevel::Strict, None),
    "collectgarbage" => Constraint::AtMost(SandboxLevel::Strict, None),
    "dofile"         => Constraint::AtMost(SandboxLevel::Strict, None),
    "error"          => Constraint::AtMost(SandboxLevel::Strict, None),
    "gcinfo"         => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
    "getfenv"        => Constraint::AtMost(SandboxLevel::Strict, None),
    "getmetatable"   => Constraint::AtMost(SandboxLevel::Strict, None),
    "ipairs"         => Constraint::AtMost(SandboxLevel::Strict, None),
    "load"           => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)),
    "loadfile"       => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
    "loadstring"     => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)),
    "module"         => Constraint::AtMost(SandboxLevel::Strict, None),
    "newproxy"       => Constraint::AtMost(SandboxLevel::Strict, None),
    "next"           => Constraint::AtMost(SandboxLevel::Strict, None),
    "pairs"          => Constraint::AtMost(SandboxLevel::Strict, None),
    "pcall"          => Constraint::AtMost(SandboxLevel::Strict, None),
    "print"          => Constraint::AtMost(SandboxLevel::Strict, None),
    "rawequal"       => Constraint::AtMost(SandboxLevel::Strict, None),
    "rawget"         => Constraint::AtMost(SandboxLevel::Strict, None),
    "rawlen"         => Constraint::AtMost(SandboxLevel::Strict, None),
    "rawset"         => Constraint::AtMost(SandboxLevel::Strict, None),
    "require"        => Constraint::AtMost(SandboxLevel::Strict, None),
    "select"         => Constraint::AtMost(SandboxLevel::Strict, None),
    "setfenv"        => Constraint::AtMost(SandboxLevel::Strict, None),
    "setmetatable"   => Constraint::AtMost(SandboxLevel::Strict, None),
    "tonumber"       => Constraint::AtMost(SandboxLevel::Strict, None),
    "tostring"       => Constraint::AtMost(SandboxLevel::Strict, None),
    "type"           => Constraint::AtMost(SandboxLevel::Strict, None),
    "unpack"         => Constraint::AtMost(SandboxLevel::Strict, None),
    "xpcall"         => Constraint::AtMost(SandboxLevel::Strict, None),

    // Tables
    "bit" => Constraint::Table(phf_map!{
        "arshift" => Constraint::AtMost(SandboxLevel::Strict, None),
        "band"    => Constraint::AtMost(SandboxLevel::Strict, None),
        "bnot"    => Constraint::AtMost(SandboxLevel::Strict, None),
        "bor"     => Constraint::AtMost(SandboxLevel::Strict, None),
        "bswap"   => Constraint::AtMost(SandboxLevel::Strict, None),
        "bxor"    => Constraint::AtMost(SandboxLevel::Strict, None),
        "lshift"  => Constraint::AtMost(SandboxLevel::Strict, None),
        "rol"     => Constraint::AtMost(SandboxLevel::Strict, None),
        "ror"     => Constraint::AtMost(SandboxLevel::Strict, None),
        "rshift"  => Constraint::AtMost(SandboxLevel::Strict, None),
        "tobit"   => Constraint::AtMost(SandboxLevel::Strict, None),
        "tohex"   => Constraint::AtMost(SandboxLevel::Strict, None),
    }),
    "coroutine" => Constraint::Table(phf_map!{
        "create"      => Constraint::AtMost(SandboxLevel::Strict, None),
        "isyieldable" => Constraint::AtMost(SandboxLevel::Strict, None),
        "resume"      => Constraint::AtMost(SandboxLevel::Strict, None),
        "running"     => Constraint::AtMost(SandboxLevel::Strict, None),
        "status"      => Constraint::AtMost(SandboxLevel::Strict, None),
        "wrap"        => Constraint::AtMost(SandboxLevel::Strict, None),
        "yield"       => Constraint::AtMost(SandboxLevel::Strict, None),
    }),
    "debug" => Constraint::Table(phf_map!{
        "debug"        => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "getfenv"      => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "gethook"      => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "getinfo"      => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "getlocal"     => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "getmetatable" => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "getregistry"  => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "getupvalue"   => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "getuservalue" => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "setfenv"      => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "sethook"      => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "setlocal"     => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "setmetatable" => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "setupvalue"   => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "setuservalue" => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "traceback"    => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "upvalueid"    => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "upvaluejoin"  => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
    }),
    "ffi" => Constraint::Table(phf_map!{
        "C"        => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::Nil)),
        "abi"      => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "alignof"  => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "arch"     => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::Nil)),
        "cast"     => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "cdef"     => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "copy"     => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "errno"    => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "fill"     => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "gc"       => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "istype"   => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "load"     => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "metatype" => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "new"      => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "offsetof" => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "os"       => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::Nil)),
        "sizeof"   => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "string"   => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "typeinfo" => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "typeof"   => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
    }),
    "io" => Constraint::Table(phf_map!{
        "close"   => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)),
        "flush"   => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)),
        "input"   => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)), // TODO(kcza): replace with custom one which only allows in current dir!
        "lines"   => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)), // TODO(kcza): replace with custom one which only allows in current dir!
        "open"    => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)), // TODO(kcza): replace with custom one which only allows in current dir!
        "output"  => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)), // TODO(kcza): replace with custom one which only allows in current dir!
        "popen"   => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "read"    => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)), // TODO(kcza): replace with custom one which only allows in current dir!
        "stderr"  => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::Nil)),
        "stdin"   => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::Nil)),
        "stdout"  => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::Nil)),
        "tmpfile" => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)), // TODO(kcza): replace with custom one which only allows in current dir!
        "type"    => Constraint::AtMost(SandboxLevel::Strict, None),
        "write"   => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)), // TODO(kcza): replace with custom one which only allows in current dir!
    }),
    "jit" => Constraint::Table(phf_map!{
        "arch"        => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::Nil)),
        "attach"      => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)),
        "flush"       => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)),
        "off"         => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)),
        "on"          => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)),
        "opt"         => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::Nil)),
        "os"          => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::Nil)),
        "prngstate"   => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "security"    => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "status"      => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "version"     => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::Nil)),
        "version_num" => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::Nil)),
    }),
    "math" => Constraint::Table(phf_map!{
        "abs"        => Constraint::AtMost(SandboxLevel::Strict, None),
        "acos"       => Constraint::AtMost(SandboxLevel::Strict, None),
        "asin"       => Constraint::AtMost(SandboxLevel::Strict, None),
        "atan"       => Constraint::AtMost(SandboxLevel::Strict, None),
        "atan2"      => Constraint::AtMost(SandboxLevel::Strict, None),
        "ceil"       => Constraint::AtMost(SandboxLevel::Strict, None),
        "cos"        => Constraint::AtMost(SandboxLevel::Strict, None),
        "cosh"       => Constraint::AtMost(SandboxLevel::Strict, None),
        "deg"        => Constraint::AtMost(SandboxLevel::Strict, None),
        "exp"        => Constraint::AtMost(SandboxLevel::Strict, None),
        "floor"      => Constraint::AtMost(SandboxLevel::Strict, None),
        "fmod"       => Constraint::AtMost(SandboxLevel::Strict, None),
        "frexp"      => Constraint::AtMost(SandboxLevel::Strict, None),
        "huge"       => Constraint::AtMost(SandboxLevel::Strict, None),
        "ldexp"      => Constraint::AtMost(SandboxLevel::Strict, None),
        "log"        => Constraint::AtMost(SandboxLevel::Strict, None),
        "log10"      => Constraint::AtMost(SandboxLevel::Strict, None),
        "max"        => Constraint::AtMost(SandboxLevel::Strict, None),
        "min"        => Constraint::AtMost(SandboxLevel::Strict, None),
        "modf"       => Constraint::AtMost(SandboxLevel::Strict, None),
        "pi"         => Constraint::AtMost(SandboxLevel::Strict, None),
        "pow"        => Constraint::AtMost(SandboxLevel::Strict, None),
        "rad"        => Constraint::AtMost(SandboxLevel::Strict, None),
        "random"     => Constraint::AtMost(SandboxLevel::Strict, None),
        "randomseed" => Constraint::AtMost(SandboxLevel::Strict, None),
        "sin"        => Constraint::AtMost(SandboxLevel::Strict, None),
        "sinh"       => Constraint::AtMost(SandboxLevel::Strict, None),
        "sqrt"       => Constraint::AtMost(SandboxLevel::Strict, None),
        "tan"        => Constraint::AtMost(SandboxLevel::Strict, None),
        "tanh"       => Constraint::AtMost(SandboxLevel::Strict, None),
    }),
    "os" => Constraint::Table(phf_map!{
        "clock"     => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)),
        "date"      => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)),
        "difftime"  => Constraint::AtMost(SandboxLevel::Strict, None),
        "execute"   => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "exit"      => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::ErrFunc("os.exit"))),
        "getenv"    => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "remove"    => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)), // TODO(kcza): replace with sandboxed one!
        "rename"    => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)), // TODO(kcza): replace with sandboxed one!
        "setlocale" => Constraint::AtMost(SandboxLevel::Unrestricted, Some(Replacement::NilFunc)),
        "time"      => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)),
        "tmpname"   => Constraint::AtMost(SandboxLevel::Standard, Some(Replacement::NilFunc)),
    }),
    "package" => Constraint::Table(phf_map!{
        "config"     => Constraint::AtMost(SandboxLevel::Strict, None),
        "cpath"      => Constraint::AtMost(SandboxLevel::Strict, None),
        "loaded"     => Constraint::AtMost(SandboxLevel::Strict, None),
        "loaders"    => Constraint::AtMost(SandboxLevel::Strict, None),
        "loadlib"    => Constraint::AtMost(SandboxLevel::Strict, None),
        "path"       => Constraint::AtMost(SandboxLevel::Strict, None),
        "preload"    => Constraint::AtMost(SandboxLevel::Strict, None),
        "searchers"  => Constraint::AtMost(SandboxLevel::Strict, None),
        "searchpath" => Constraint::AtMost(SandboxLevel::Strict, None),
        "seeall"     => Constraint::AtMost(SandboxLevel::Strict, None),
    }),
    "string" => Constraint::Table(phf_map!{
        "byte"    => Constraint::AtMost(SandboxLevel::Strict, None),
        "char"    => Constraint::AtMost(SandboxLevel::Strict, None),
        "dump"    => Constraint::AtMost(SandboxLevel::Strict, None),
        "find"    => Constraint::AtMost(SandboxLevel::Strict, None),
        "format"  => Constraint::AtMost(SandboxLevel::Strict, None),
        "gmatch"  => Constraint::AtMost(SandboxLevel::Strict, None),
        "gsub"    => Constraint::AtMost(SandboxLevel::Strict, None),
        "len"     => Constraint::AtMost(SandboxLevel::Strict, None),
        "lower"   => Constraint::AtMost(SandboxLevel::Strict, None),
        "match"   => Constraint::AtMost(SandboxLevel::Strict, None),
        "rep"     => Constraint::AtMost(SandboxLevel::Strict, None),
        "reverse" => Constraint::AtMost(SandboxLevel::Strict, None),
        "sub"     => Constraint::AtMost(SandboxLevel::Strict, None),
        "upper"   => Constraint::AtMost(SandboxLevel::Strict, None),
    }),
    "table" => Constraint::Table(phf_map!{
        "clear"    => Constraint::AtMost(SandboxLevel::Strict, None),
        "clone"    => Constraint::AtMost(SandboxLevel::Strict, None),
        "concat"   => Constraint::AtMost(SandboxLevel::Strict, None),
        "foreach"  => Constraint::AtMost(SandboxLevel::Strict, None),
        "foreachi" => Constraint::AtMost(SandboxLevel::Strict, None),
        "getn"     => Constraint::AtMost(SandboxLevel::Strict, None),
        "insert"   => Constraint::AtMost(SandboxLevel::Strict, None),
        "isarray"  => Constraint::AtMost(SandboxLevel::Strict, None),
        "isempty"  => Constraint::AtMost(SandboxLevel::Strict, None),
        "maxn"     => Constraint::AtMost(SandboxLevel::Strict, None),
        "move"     => Constraint::AtMost(SandboxLevel::Strict, None),
        "new"      => Constraint::AtMost(SandboxLevel::Strict, None),
        "nkeys"    => Constraint::AtMost(SandboxLevel::Strict, None),
        "pack"     => Constraint::AtMost(SandboxLevel::Strict, None),
        "remove"   => Constraint::AtMost(SandboxLevel::Strict, None),
        "sort"     => Constraint::AtMost(SandboxLevel::Strict, None),
        "unpack"   => Constraint::AtMost(SandboxLevel::Strict, None),
    }),
    "utf8" => Constraint::Table(phf_map!{
        "char"        => Constraint::AtMost(SandboxLevel::Strict, None),
        "charpattern" => Constraint::AtMost(SandboxLevel::Strict, None),
        "codes"       => Constraint::AtMost(SandboxLevel::Strict, None),
        "codepoint"   => Constraint::AtMost(SandboxLevel::Strict, None),
        "len"         => Constraint::AtMost(SandboxLevel::Strict, None),
        "offset"      => Constraint::AtMost(SandboxLevel::Strict, None),
    }),
};

#[cfg(test)]
mod test {
    use crate::Context;

    use super::*;
    use std::error::Error;

    #[test]
    fn all_globals_constained() -> Result<(), Box<dyn Error>> {
        let ctx = {
            let mut ctx = Context::test_new();
            ctx.lua_params_mut().set_sandbox_level(SandboxLevel::Unsound);
            ctx
        };
        let ext_state = ctx.extension_state()?;
        let lua = ext_state.lua();

        let mut uncovered = Vec::new();
        for global in lua.globals().pairs::<String, Value>() {
            let (k, v) = global.unwrap();
            let constraint = {
                let found = CONSTRAINTS.get(&k);
                if found.is_none() {
                    uncovered.push(k);
                    continue;
                }
                found.unwrap()
            };

            match (constraint, &v) {
                (Constraint::Table(child_levels), Value::Table(t)) => {
                    for entry in t.clone().pairs::<String, Value>() {
                        let (k2, _) = entry?;
                        let constraint = {
                            let found = child_levels.get(&k2);
                            if found.is_none() {
                                uncovered.push(format!("{k}.{k2}"));
                                continue;
                            }
                            found.unwrap()
                        };
                        assert!(matches!(constraint, Constraint::AtMost(..)));
                    }
                }
                (Constraint::Table { .. }, _) => panic!("constraint expected table in {k}"),
                _ => {}
            }
        }
        uncovered.sort();
        assert!(
            uncovered.is_empty(),
            "uncovered globals: {}",
            uncovered.join(", ")
        );

        Ok(())
    }

    #[test]
    fn all_constraints_used() -> Result<(), Box<dyn Error>> {
        let ctx = {
            let mut ctx = Context::test_new();
            ctx.lua_params_mut().set_sandbox_level(SandboxLevel::Unsound);
            ctx
        };
        let ext_state = ctx.extension_state()?;
        let lua = ext_state.lua();
        let globals = lua.globals();

        let mut unused = Vec::new();
        for (k, c) in &CONSTRAINTS {
            match (globals.get::<&str, Value>(k), c) {
                (Ok(Value::Nil), _) => unused.push(k.to_string()),
                (Ok(Value::Table(t)), Constraint::Table(child_levels)) => {
                    for (k2, c2) in child_levels {
                        assert!(matches!(c2, Constraint::AtMost(..)));
                        if let Value::Nil = t.get(*k2).unwrap() {
                            unused.push(format!("{k}.{k2}"));
                        }
                    }
                }
                (Err(e), _) => panic!("{e}"),
                _ => {}
            }
        }
        unused.sort();
        assert!(
            unused.is_empty(),
            "unused constraints: {}",
            unused.join(", ")
        );

        Ok(())
    }

    #[test]
    fn replacements_maintain_types() {
        fn check_replacements(
            lua: &Lua,
            table: Table,
            constraints: &Map<&'static str, Constraint>,
        ) -> Result<(), Box<dyn Error>> {
            for entry in table.pairs() {
                let (k, v): (String, Value) = entry?;
                match &constraints[&k] {
                    Constraint::AtMost(_, None) => {}
                    Constraint::AtMost(_, Some(r)) => {
                        for level in SandboxLevel::input_levels() {
                            match r.make(lua, level)?.type_name() {
                                "nil" => assert_ne!(
                                    v.type_name(),
                                    "function",
                                    "for a function for {k:?}"
                                ),
                                name => assert_eq!(v.type_name(), name, "types for {k:?} differ"),
                            }
                        }
                    }
                    Constraint::Table(c) => {
                        if let Value::Table(t) = v {
                            check_replacements(lua, t, c)?
                        }
                    }
                }
            }

            Ok(())
        }

        let lua = unsafe { Lua::unsafe_new() };
        check_replacements(&lua, lua.globals(), &CONSTRAINTS).unwrap()
    }

    #[test]
    fn replacements_provided_correctly() {
        fn check_replacements(constraints: &Map<&'static str, Constraint>) {
            for (k, c) in constraints {
                match c {
                    Constraint::AtMost(SandboxLevel::Strict, Some(_)) => {
                        panic!("{k} has unused replacement function")
                    }
                    Constraint::AtMost(_, Some(_)) => {}
                    Constraint::AtMost(SandboxLevel::Strict, None) => {}
                    Constraint::AtMost(_, None) => panic!("{k} has no replacement"),
                    Constraint::Table(c) => check_replacements(c),
                }
            }
        }

        check_replacements(&CONSTRAINTS)
    }
}

// TODO(kcza): test application of restrictions by sample
// TODO(kcza): check the currect values are computed for specific cases!
