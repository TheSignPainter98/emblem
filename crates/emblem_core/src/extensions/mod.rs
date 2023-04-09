mod lua_restrictions;

use crate::context::{ResourceLimit, SandboxLevel};
use mlua::{Error as MLuaError, HookTriggers, Lua, Result as MLuaResult, Table, Value, TableExt};
use std::{cell::RefMut, fmt::Display, sync::Arc};

macro_rules! emblem_registry_key {
    ($name:literal) => {
        concat!("__emblem_", $name)
    };
}

static STD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/yue/std.luac"));
const EVENT_LISTENERS_RKEY: &str = emblem_registry_key!("events"); // TODO(kcza): error if non-function, non-callable used as listener

#[derive(Copy, Clone, Default)]
pub struct ExtensionStateBuilder {
    pub sandbox_level: SandboxLevel,
    pub max_mem: ResourceLimit,
    pub max_steps: ResourceLimit,
}

impl ExtensionStateBuilder {
    pub fn build(self) -> MLuaResult<ExtensionState> {
        let lua = match self.sandbox_level {
            SandboxLevel::Unrestricted => unsafe { Lua::unsafe_new() },
            _ => Lua::new(),
        };

        lua.set_app_data(ExtensionData::new());

        self.insert_safety_hook(&lua)?;
        lua_restrictions::restrict(&lua, self.sandbox_level)?;
        self.setup_event_listeners(&lua)?;

        // TODO(kcza): set args

        lua.load(STD).exec()?;

        Ok(ExtensionState { lua })
    }

    fn insert_safety_hook(&self, lua: &Lua) -> MLuaResult<()> {
        const INSTRUCTION_INTERVAL: u32 = 64;

        let max_mem = self.max_steps.into();
        let max_steps: u32 = match self.max_mem.try_into() {
            Ok(m) => m,
            Err(e) => return Err(MLuaError::ExternalError(Arc::new(e))),
        };

        lua.set_hook(
            HookTriggers::every_nth_instruction(INSTRUCTION_INTERVAL),
            move |lua, _debug| {
                if lua.used_memory() >= max_mem {
                    return Err(MLuaError::SafetyError("too much memory used".into()));
                }

                let mut data: RefMut<'_, ExtensionData> = lua.app_data_mut().unwrap();
                data.curr_steps += INSTRUCTION_INTERVAL;
                if data.curr_steps > max_steps {
                    return Err(MLuaError::SafetyError("too many steps".into()));
                }

                Ok(())
            },
        )
    }

    fn setup_event_listeners(&self, lua: &Lua) -> MLuaResult<()> {
        lua.set_named_registry_value(EVENT_LISTENERS_RKEY, {
            let listeners = lua.create_table_with_capacity(0, 3)?;
            for event in [Event::IterStart, Event::IterEnd, Event::Done] {
                listeners.set(event.name(), lua.create_table()?)?;
            }
            listeners
        })
    }
}

pub struct ExtensionState {
    lua: Lua,
    // phantom: PhantomData<&'em bool>, // TODO(kcza): use phantomdata to plumb the arena through
    // and fix the lifetimes
}

impl ExtensionState {
    pub fn handle(&self, event: Event) -> Result<(), MLuaError> {
        let event_listeners: Table = self.lua.named_registry_value(EVENT_LISTENERS_RKEY)?;
        let listeners = match event_listeners.get::<_, Option<Table>>(event.name())? {
            Some(listeners) => listeners,
            None => panic!("internal error: {event} event has no listeners table"),
        };
        for listener in listeners.sequence_values::<Value>() {
            let event_data = self.event_data(event);
            match listener? {
                Value::Function(f) => f.call(event_data)?,
                Value::Table(t) => t.call(event_data)?,
                v => return Err(MLuaError::RuntimeError(format!("non-callable listener (got a {}) found when handling {event} event", v.type_name()))),
            }
        }

        Ok(())
    }

    fn event_data(&self, event: Event) -> MLuaResult<Value> {
        // TODO(kcza): get event data
        let data = match event {
            Event::IterStart | Event::IterEnd => {
                let event = self.lua.create_table_with_capacity(0, 1)?;
                event.set("iter", self.curr_iter())?;
                event
            }
            Event::Done => self.lua.create_table()?,
        };
        Ok(Value::Table(data))
    }

    pub(crate) fn curr_iter(&self) -> u32 {
        self.lua
            .app_data_ref::<ExtensionData>()
            .expect("lua app data not set")
            .curr_iter
    }

    pub(crate) fn reiter_requested(&self) -> bool {
        self.lua
            .app_data_ref::<ExtensionData>()
            .expect("lua app data not set")
            .reiter_requested
    }

    pub(crate) fn increment_iter_count(&mut self) {
        self.lua
            .app_data_mut::<ExtensionData>()
            .expect("lua app data not set")
            .curr_iter += 1;
    }
}

#[derive(Default)]
struct ExtensionData {
    curr_steps: u32,
    curr_iter: u32,
    reiter_requested: bool,
}

impl ExtensionData {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(Copy, Clone)]
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
            let builder = ExtensionStateBuilder {
                sandbox_level: level,
                max_mem: ResourceLimit::Unlimited,
                max_steps: ResourceLimit::Unlimited,
            };
            yuescript::Tester::new().test(builder.build().unwrap().lua);
        }
    }
}
