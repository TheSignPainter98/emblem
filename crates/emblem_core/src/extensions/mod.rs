mod sandbox;

use crate::context::{MemoryLimit, SandboxLevel};
use derive_new::new;
use mlua::{Error as MLuaError, Function, Lua, Table};
use std::fmt::Display;

macro_rules! emblem_registry_key {
    ($name:literal) => {
        concat!("__emblem_", $name)
    };
}

static STD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/yue/std.luac"));
const EVENT_LISTENERS_RKEY: &str = emblem_registry_key!("events");

#[derive(new)]
pub struct ExtensionStateBuilder {
    sandbox_level: SandboxLevel,
    max_mem: MemoryLimit,
}

impl ExtensionStateBuilder {
    pub fn build(&self) -> Result<ExtensionState, MLuaError> {
        let lua = match self.sandbox_level {
            SandboxLevel::Unrestricted => unsafe { Lua::unsafe_new() },
            _ => Lua::new(),
        };

        lua.set_named_registry_value(EVENT_LISTENERS_RKEY, {
            let listeners = lua.create_table_with_capacity(0, 3)?;
            for event in [Event::IterStart, Event::IterEnd, Event::Done] {
                listeners.set(event.name(), lua.create_table()?)?;
            }
            listeners
        })?;

        lua.load(STD).exec()?;
        sandbox::sandbox_global(&lua, self.sandbox_level)?;

        // TODO(kcza): set max mem hook
        // TODO(kcza): set max steps hook
        // TODO(kcza): set args

        Ok(ExtensionState { lua })
    }
}

pub struct ExtensionState {
    lua: Lua,
}

impl ExtensionState {
    pub fn handle(&self, event: Event) -> Result<(), MLuaError> {
        let event_listeners: Table = self.lua.named_registry_value(EVENT_LISTENERS_RKEY)?;
        let listeners = match event_listeners.get::<_, Option<Table>>(event.name())? {
            Some(listeners) => listeners,
            None => panic!("internal error: event '{event}' has no listeners table"),
        };
        for listener in listeners.sequence_values::<Function>() {
            listener.unwrap().call::<_, ()>(())?;
        }

        Ok(())
    }
}

pub enum Event {
    IterStart,
    IterEnd,
    Done,
}

impl Event {
    pub fn name(&self) -> &'static str {
        match self {
            Self::IterStart => "iter-start",
            Self::IterEnd => "iter-end",
            Self::Done => "done",
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name().fmt(f)
    }
}

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

    panic!(
        "{}",
        STD.iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .concat()
    );
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn std_tests() {
        for level in [
            SandboxLevel::Unrestricted,
            SandboxLevel::Standard,
            SandboxLevel::Strict,
        ] {
            yuescript::Tester::new().test(ExtensionState::new(level).unwrap().lua);
        }
    }
}
