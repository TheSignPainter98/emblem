mod env_extras;
mod global_sandboxing;
mod preload_decls;
mod preload_sandboxing;

use crate::context::{ResourceLimit, SandboxLevel};
use mlua::{
    Error as MLuaError, HookTriggers, Lua, MetaMethod, Result as MLuaResult, Table, TableExt, Value,
};
use std::{cell::RefMut, fmt::Display, sync::Arc};

#[cfg(test)]
use mlua::AsChunk;

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
    pub max_mem: ResourceLimit<usize>,
    pub max_steps: ResourceLimit<u32>,
}

impl ExtensionStateBuilder {
    pub fn build(self) -> MLuaResult<ExtensionState> {
        let lua = if self.sandbox_level <= SandboxLevel::Unrestricted {
            unsafe { Lua::unsafe_new() }
        } else {
            Lua::new()
        };

        lua.set_app_data(ExtensionData::new());

        preload_sandboxing::restrict_preload(&lua, self.sandbox_level)?;
        env_extras::import_extras(&lua)?;
        global_sandboxing::restrict_globals(&lua, self.sandbox_level)?;

        self.insert_safety_hook(&lua)?;
        self.setup_event_listeners(&lua)?;

        // TODO(kcza): set args

        lua.load(STD).exec()?;

        Ok(ExtensionState { lua })
    }

    fn insert_safety_hook(&self, lua: &Lua) -> MLuaResult<()> {
        const INSTRUCTION_INTERVAL: u32 = 64;

        let max_mem = self
            .max_mem
            .try_into()
            .map_err(|e| MLuaError::ExternalError(Arc::new(e)))?;
        let max_steps = self
            .max_steps
            .try_into()
            .map_err(|e| MLuaError::ExternalError(Arc::new(e)))?;

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
            for event in Event::events() {
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
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    pub fn add_listener(&self, event: Event, listener: Value) -> MLuaResult<()> {
        if !callable(&listener) {
            return Err(MLuaError::RuntimeError(format!(
                "non-callable listener {} found when handling {event} event",
                listener.type_name()
            )));
        }

        let listeners: Table = self.lua.named_registry_value(EVENT_LISTENERS_RKEY)?;
        let event_listeners = match listeners.get::<_, Option<Table>>(event.name())? {
            Some(ls) => ls,
            None => panic!("internal error: {event} event has no listeners table"),
        };

        event_listeners.push(listener)
    }

    pub fn handle(&self, event: Event) -> MLuaResult<()> {
        let listeners: Table = self.lua.named_registry_value(EVENT_LISTENERS_RKEY)?;
        let event_listeners = match listeners.get::<_, Option<Table>>(event.name())? {
            Some(ls) => ls,
            None => panic!("internal error: {event} event has no listeners table"),
        };

        for listener in event_listeners.sequence_values::<Value>() {
            self.call_listener(listener?, event)?;
        }

        Ok(())
    }

    fn call_listener(&self, listener: Value, event: Event) -> MLuaResult<()> {
        if let Value::Function(f) = listener {
            f.call(self.event_data(event))
        } else {
            self.call_listener_method(listener, event)
        }
    }

    fn call_listener_method(&self, listener: Value, event: Event) -> MLuaResult<()> {
        let type_name = listener.type_name();

        match listener.clone() {
            Value::Function(f) => return f.call(self.event_data(event)?),
            Value::Table(t) => return t.call(self.event_data(event)?),
            Value::UserData(u) => {
                if let Ok(mt) = u.get_metatable() {
                    if let Ok(m) = mt.get::<_, Value>(MetaMethod::Call.name()) {
                        match m {
                            Value::Function(f) => {
                                return f.call((listener, self.event_data(event)?))
                            }
                            Value::Table(t) => return t.call((listener, self.event_data(event)?)),
                            Value::UserData(_) => return self.call_listener_method(m, event),
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }

        Err(MLuaError::RuntimeError(format!(
            "non-callable listener (got a {type_name}) found when handling {event} event",
        )))
    }

    fn event_data(&self, event: Event) -> MLuaResult<Value> {
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

    pub(crate) fn reset_reiter_request(&self) {
        self.lua
            .app_data_mut::<ExtensionData>()
            .unwrap()
            .reset_reiter_request();
    }

    pub(crate) fn increment_iter_count(&mut self) {
        self.lua
            .app_data_mut::<ExtensionData>()
            .expect("lua app data not set")
            .curr_iter += 1;
    }
}

#[cfg(test)]
impl ExtensionState {
    pub fn run<'lua, C: AsChunk<'lua> + ?Sized>(&'lua self, chunk: &'lua C) -> MLuaResult<()> {
        self.lua.load(chunk).exec()
    }
}

fn callable(value: &Value) -> bool {
    match value {
        Value::Function(_) => true,
        Value::Table(t) => {
            if let Some(mt) = t.get_metatable() {
                if let Ok(c) = mt.raw_get(MetaMethod::Call.name()) {
                    return callable(&c);
                }
            }
            false
        }
        Value::UserData(u) => {
            if let Ok(mt) = u.get_metatable() {
                if let Ok(c) = mt.get(MetaMethod::Call.name()) {
                    return callable(&c);
                }
            }
            false
        }
        _ => false,
    }
}

#[derive(Default)]
pub(crate) struct ExtensionData {
    curr_steps: u32,
    curr_iter: u32,
    reiter_requested: bool,
}

impl ExtensionData {
    fn new() -> Self {
        Self::default()
    }

    #[allow(unused)]
    pub(crate) fn request_reiter(&mut self) {
        self.reiter_requested = true;
    }

    pub(crate) fn reset_reiter_request(&mut self) {
        self.reiter_requested = false;
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

    fn events() -> &'static [Event] {
        &[Self::IterStart, Self::IterEnd, Self::Done]
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
        for level in SandboxLevel::input_levels() {
            let builder = ExtensionStateBuilder {
                sandbox_level: level,
                max_mem: ResourceLimit::Unlimited,
                max_steps: ResourceLimit::Unlimited,
            };
            yuescript::Tester::new().test(builder.build().unwrap().lua);
        }
    }

    // TODO(kcza): test sandboxing application
    // TODO(kcza): test step limits
    // TODO(kcza): test memory limits
}
