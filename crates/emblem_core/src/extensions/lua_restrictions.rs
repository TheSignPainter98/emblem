use crate::context::SandboxLevel;
use mlua::{Error as MLuaError, Lua, Result as MLuaResult, Table, Value};
use phf::{phf_map, Map};

pub(crate) fn restrict(lua: &Lua, level: SandboxLevel) -> Result<(), MLuaError> {
    restrict_table(level, lua.globals(), &CONSTRAINTS)
}

fn restrict_table(
    level: SandboxLevel,
    table: Table,
    constraints: &Map<&'static str, Constraint>,
) -> MLuaResult<()> {
    let mut to_remove = Vec::new();
    for entry in table.pairs() {
        let (k, v): (String, Value) = entry?;
        match &constraints[&k] {
            Constraint::AtLeast(l) => {
                if level < *l {
                    to_remove.push(k)
                }
            }
            Constraint::Table {
                at_least: l,
                child_levels,
            } => {
                if level < *l {
                    to_remove.push(k)
                } else if let Value::Table(t) = v {
                    restrict_table(level, t, child_levels)?;
                } else {
                    panic!("internal error: expected table in {k}");
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
enum Constraint {
    AtLeast(SandboxLevel),
    Table {
        at_least: SandboxLevel,
        child_levels: Map<&'static str, Constraint>,
    },
}

static CONSTRAINTS: Map<&'static str, Constraint> = phf_map! {
    // Values
    "_G"       => Constraint::AtLeast(SandboxLevel::Strict),
    "_VERSION" => Constraint::AtLeast(SandboxLevel::Strict),

    // Functions
    "assert"         => Constraint::AtLeast(SandboxLevel::Unrestricted),
    "collectgarbage" => Constraint::AtLeast(SandboxLevel::Strict),
    "dofile"         => Constraint::AtLeast(SandboxLevel::Strict),
    "error"          => Constraint::AtLeast(SandboxLevel::Strict),
    "gcinfo"         => Constraint::AtLeast(SandboxLevel::Unrestricted),
    "getfenv"        => Constraint::AtLeast(SandboxLevel::Strict),
    "getmetatable"   => Constraint::AtLeast(SandboxLevel::Strict),
    "ipairs"         => Constraint::AtLeast(SandboxLevel::Strict),
    "load"           => Constraint::AtLeast(SandboxLevel::Standard),
    "loadfile"       => Constraint::AtLeast(SandboxLevel::Unrestricted),
    "loadstring"     => Constraint::AtLeast(SandboxLevel::Standard),
    "module"         => Constraint::AtLeast(SandboxLevel::Strict),
    "newproxy"       => Constraint::AtLeast(SandboxLevel::Strict),
    "next"           => Constraint::AtLeast(SandboxLevel::Strict),
    "pairs"          => Constraint::AtLeast(SandboxLevel::Strict),
    "pcall"          => Constraint::AtLeast(SandboxLevel::Strict),
    "print"          => Constraint::AtLeast(SandboxLevel::Strict),
    "rawequal"       => Constraint::AtLeast(SandboxLevel::Strict),
    "rawget"         => Constraint::AtLeast(SandboxLevel::Strict),
    "rawlen"         => Constraint::AtLeast(SandboxLevel::Strict),
    "rawset"         => Constraint::AtLeast(SandboxLevel::Strict),
    "require"        => Constraint::AtLeast(SandboxLevel::Strict),
    "select"         => Constraint::AtLeast(SandboxLevel::Strict),
    "setfenv"        => Constraint::AtLeast(SandboxLevel::Strict),
    "setmetatable"   => Constraint::AtLeast(SandboxLevel::Strict),
    "tonumber"       => Constraint::AtLeast(SandboxLevel::Strict),
    "tostring"       => Constraint::AtLeast(SandboxLevel::Strict),
    "type"           => Constraint::AtLeast(SandboxLevel::Strict),
    "unpack"         => Constraint::AtLeast(SandboxLevel::Strict),
    "xpcall"         => Constraint::AtLeast(SandboxLevel::Strict),

    // Tables
    "bit" => Constraint::Table { at_least: SandboxLevel::Strict, child_levels: phf_map!{
        "arshift" => Constraint::AtLeast(SandboxLevel::Strict),
        "band"    => Constraint::AtLeast(SandboxLevel::Strict),
        "bnot"    => Constraint::AtLeast(SandboxLevel::Strict),
        "bor"     => Constraint::AtLeast(SandboxLevel::Strict),
        "bswap"   => Constraint::AtLeast(SandboxLevel::Strict),
        "bxor"    => Constraint::AtLeast(SandboxLevel::Strict),
        "lshift"  => Constraint::AtLeast(SandboxLevel::Strict),
        "rol"     => Constraint::AtLeast(SandboxLevel::Strict),
        "ror"     => Constraint::AtLeast(SandboxLevel::Strict),
        "rshift"  => Constraint::AtLeast(SandboxLevel::Strict),
        "tobit"   => Constraint::AtLeast(SandboxLevel::Strict),
        "tohex"   => Constraint::AtLeast(SandboxLevel::Strict),
    }},
    "coroutine" => Constraint::Table{ at_least: SandboxLevel::Strict, child_levels: phf_map!{
        "create"      => Constraint::AtLeast(SandboxLevel::Strict),
        "isyieldable" => Constraint::AtLeast(SandboxLevel::Strict),
        "resume"      => Constraint::AtLeast(SandboxLevel::Strict),
        "running"     => Constraint::AtLeast(SandboxLevel::Strict),
        "status"      => Constraint::AtLeast(SandboxLevel::Strict),
        "wrap"        => Constraint::AtLeast(SandboxLevel::Strict),
        "yield"       => Constraint::AtLeast(SandboxLevel::Strict),
    }},
    "debug" => Constraint::Table{ at_least: SandboxLevel::Unrestricted, child_levels: phf_map!{
        "debug"        => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "getfenv"      => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "gethook"      => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "getinfo"      => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "getlocal"     => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "getmetatable" => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "getregistry"  => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "getupvalue"   => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "getuservalue" => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "setfenv"      => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "sethook"      => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "setlocal"     => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "setmetatable" => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "setupvalue"   => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "setuservalue" => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "traceback"    => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "upvalueid"    => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "upvaluejoin"  => Constraint::AtLeast(SandboxLevel::Unrestricted),
    }},
    "ffi" => Constraint::Table{ at_least: SandboxLevel::Unrestricted, child_levels: phf_map!{
        "C"        => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "abi"      => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "alignof"  => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "arch"     => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "cast"     => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "cdef"     => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "copy"     => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "errno"    => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "fill"     => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "gc"       => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "istype"   => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "load"     => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "metatype" => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "new"      => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "offsetof" => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "os"       => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "sizeof"   => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "string"   => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "typeinfo" => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "typeof"   => Constraint::AtLeast(SandboxLevel::Unrestricted),
    }},
    "io" => Constraint::Table{ at_least: SandboxLevel::Standard, child_levels: phf_map!{
        "close"   => Constraint::AtLeast(SandboxLevel::Standard),
        "flush"   => Constraint::AtLeast(SandboxLevel::Standard),
        "input"   => Constraint::AtLeast(SandboxLevel::Standard),
        "lines"   => Constraint::AtLeast(SandboxLevel::Standard), // TODO(kcza): replace with custom one which only allows in current dir!
        "open"    => Constraint::AtLeast(SandboxLevel::Standard), // TODO(kcza): replace with custom one which only allows in current dir!
        "output"  => Constraint::AtLeast(SandboxLevel::Standard),
        "popen"   => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "read"    => Constraint::AtLeast(SandboxLevel::Standard), // TODO(kcza): replace with custom one which only allows in current dir!
        "stderr"  => Constraint::AtLeast(SandboxLevel::Standard),
        "stdin"   => Constraint::AtLeast(SandboxLevel::Standard),
        "stdout"  => Constraint::AtLeast(SandboxLevel::Standard),
        "tmpfile" => Constraint::AtLeast(SandboxLevel::Standard),
        "type"    => Constraint::AtLeast(SandboxLevel::Standard),
        "write"   => Constraint::AtLeast(SandboxLevel::Standard), // TODO(kcza): replace with custom one which only allows in current dir!
    }},
    "jit" => Constraint::Table{ at_least: SandboxLevel::Standard, child_levels: phf_map!{
        "arch"        => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "attach"      => Constraint::AtLeast(SandboxLevel::Standard),
        "flush"       => Constraint::AtLeast(SandboxLevel::Standard),
        "off"         => Constraint::AtLeast(SandboxLevel::Standard),
        "on"          => Constraint::AtLeast(SandboxLevel::Standard),
        "opt"         => Constraint::AtLeast(SandboxLevel::Standard),
        "os"          => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "prngstate"   => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "security"    => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "status"      => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "version"     => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "version_num" => Constraint::AtLeast(SandboxLevel::Unrestricted),
    }},
    "math" => Constraint::Table{ at_least: SandboxLevel::Strict, child_levels: phf_map!{
        "abs"        => Constraint::AtLeast(SandboxLevel::Strict),
        "acos"       => Constraint::AtLeast(SandboxLevel::Strict),
        "asin"       => Constraint::AtLeast(SandboxLevel::Strict),
        "atan"       => Constraint::AtLeast(SandboxLevel::Strict),
        "atan2"      => Constraint::AtLeast(SandboxLevel::Strict),
        "ceil"       => Constraint::AtLeast(SandboxLevel::Strict),
        "cos"        => Constraint::AtLeast(SandboxLevel::Strict),
        "cosh"       => Constraint::AtLeast(SandboxLevel::Strict),
        "deg"        => Constraint::AtLeast(SandboxLevel::Strict),
        "exp"        => Constraint::AtLeast(SandboxLevel::Strict),
        "floor"      => Constraint::AtLeast(SandboxLevel::Strict),
        "fmod"       => Constraint::AtLeast(SandboxLevel::Strict),
        "frexp"      => Constraint::AtLeast(SandboxLevel::Strict),
        "huge"       => Constraint::AtLeast(SandboxLevel::Strict),
        "ldexp"      => Constraint::AtLeast(SandboxLevel::Strict),
        "log"        => Constraint::AtLeast(SandboxLevel::Strict),
        "log10"      => Constraint::AtLeast(SandboxLevel::Strict),
        "max"        => Constraint::AtLeast(SandboxLevel::Strict),
        "min"        => Constraint::AtLeast(SandboxLevel::Strict),
        "modf"       => Constraint::AtLeast(SandboxLevel::Strict),
        "pi"         => Constraint::AtLeast(SandboxLevel::Strict),
        "pow"        => Constraint::AtLeast(SandboxLevel::Strict),
        "rad"        => Constraint::AtLeast(SandboxLevel::Strict),
        "random"     => Constraint::AtLeast(SandboxLevel::Strict),
        "randomseed" => Constraint::AtLeast(SandboxLevel::Strict),
        "sin"        => Constraint::AtLeast(SandboxLevel::Strict),
        "sinh"       => Constraint::AtLeast(SandboxLevel::Strict),
        "sqrt"       => Constraint::AtLeast(SandboxLevel::Strict),
        "tan"        => Constraint::AtLeast(SandboxLevel::Strict),
        "tanh"       => Constraint::AtLeast(SandboxLevel::Strict),
    }},
    "os" => Constraint::Table{ at_least: SandboxLevel::Standard, child_levels: phf_map!{
        "clock"     => Constraint::AtLeast(SandboxLevel::Standard),
        "date"      => Constraint::AtLeast(SandboxLevel::Standard),
        "difftime"  => Constraint::AtLeast(SandboxLevel::Standard),
        "execute"   => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "exit"      => Constraint::AtLeast(SandboxLevel::Standard),
        "getenv"    => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "remove"    => Constraint::AtLeast(SandboxLevel::Standard),
        "rename"    => Constraint::AtLeast(SandboxLevel::Standard), // TODO(kcza): replace with sandboxed one!
        "setlocale" => Constraint::AtLeast(SandboxLevel::Unrestricted),
        "time"      => Constraint::AtLeast(SandboxLevel::Standard),
        "tmpname"   => Constraint::AtLeast(SandboxLevel::Standard),
    }},
    "package" => Constraint::Table{ at_least: SandboxLevel::Strict, child_levels: phf_map!{
        "config"     => Constraint::AtLeast(SandboxLevel::Strict),
        "cpath"      => Constraint::AtLeast(SandboxLevel::Strict),
        "loaded"     => Constraint::AtLeast(SandboxLevel::Strict),
        "loaders"    => Constraint::AtLeast(SandboxLevel::Strict),
        "loadlib"    => Constraint::AtLeast(SandboxLevel::Strict),
        "path"       => Constraint::AtLeast(SandboxLevel::Strict),
        "preload"    => Constraint::AtLeast(SandboxLevel::Strict),
        "searchers"  => Constraint::AtLeast(SandboxLevel::Strict),
        "searchpath" => Constraint::AtLeast(SandboxLevel::Strict),
        "seeall"     => Constraint::AtLeast(SandboxLevel::Strict),
    }},
    "string" => Constraint::Table{ at_least: SandboxLevel::Strict, child_levels: phf_map!{
        "byte"    => Constraint::AtLeast(SandboxLevel::Strict),
        "char"    => Constraint::AtLeast(SandboxLevel::Strict),
        "dump"    => Constraint::AtLeast(SandboxLevel::Strict),
        "find"    => Constraint::AtLeast(SandboxLevel::Strict),
        "format"  => Constraint::AtLeast(SandboxLevel::Strict),
        "gmatch"  => Constraint::AtLeast(SandboxLevel::Strict),
        "gsub"    => Constraint::AtLeast(SandboxLevel::Strict),
        "len"     => Constraint::AtLeast(SandboxLevel::Strict),
        "lower"   => Constraint::AtLeast(SandboxLevel::Strict),
        "match"   => Constraint::AtLeast(SandboxLevel::Strict),
        "rep"     => Constraint::AtLeast(SandboxLevel::Strict),
        "reverse" => Constraint::AtLeast(SandboxLevel::Strict),
        "sub"     => Constraint::AtLeast(SandboxLevel::Strict),
        "upper"   => Constraint::AtLeast(SandboxLevel::Strict),
    }},
    "table" => Constraint::Table{ at_least: SandboxLevel::Strict, child_levels: phf_map!{
        "concat"   => Constraint::AtLeast(SandboxLevel::Strict),
        "foreach"  => Constraint::AtLeast(SandboxLevel::Strict),
        "foreachi" => Constraint::AtLeast(SandboxLevel::Strict),
        "getn"     => Constraint::AtLeast(SandboxLevel::Strict),
        "insert"   => Constraint::AtLeast(SandboxLevel::Strict),
        "maxn"     => Constraint::AtLeast(SandboxLevel::Strict),
        "move"     => Constraint::AtLeast(SandboxLevel::Strict),
        "pack"     => Constraint::AtLeast(SandboxLevel::Strict),
        "remove"   => Constraint::AtLeast(SandboxLevel::Strict),
        "sort"     => Constraint::AtLeast(SandboxLevel::Strict),
        "unpack"   => Constraint::AtLeast(SandboxLevel::Strict),
    }},
};

#[cfg(test)]
mod test {
    use super::*;
    use std::error::Error;

    #[test]
    fn all_globals_constained() {
        let lua = unsafe { Lua::unsafe_new() };

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
                (Constraint::Table { child_levels, .. }, Value::Table(t)) => {
                    for entry in t.clone().pairs::<String, Value>() {
                        let (k2, _) = entry.unwrap();
                        let constraint = {
                            let found = child_levels.get(&k2);
                            if found.is_none() {
                                uncovered.push(format!("{k}.{k2}"));
                                continue;
                            }
                            found.unwrap()
                        };
                        assert!(matches!(constraint, Constraint::AtLeast(_)));
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
    }

    #[test]
    fn all_constraints_used() -> Result<(), Box<dyn Error>> {
        let lua = unsafe { Lua::unsafe_new() };
        let globals = lua.globals();

        let mut unused = Vec::new();
        for (k, c) in &CONSTRAINTS {
            match (globals.get::<&str, Value>(k), c) {
                (Ok(Value::Nil), _) => unused.push(k.to_string()),
                (Ok(Value::Table(t)), Constraint::Table { child_levels, .. }) => {
                    for (k2, c2) in child_levels {
                        assert!(matches!(c2, Constraint::AtLeast(_)));
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
}
