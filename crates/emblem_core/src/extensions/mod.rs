mod env_extras;
mod global_sandboxing;
mod preload_decls;
mod preload_sandboxing;

use crate::{
    context::{LuaParameters, ResourceLimit, SandboxLevel},
    Context,
};
use mlua::{
    Error as MLuaError, HookTriggers, Lua, MetaMethod, Result as MLuaResult, Table, TableExt, Value,
};
use std::{cell::RefMut, fmt::Display, marker::PhantomData};

#[cfg(test)]
use mlua::AsChunk;

macro_rules! emblem_registry_key {
    ($name:literal) => {
        concat!("__emblem_", $name)
    };
}

static STD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/yue/std.luac"));
const EVENT_LISTENERS_RKEY: &str = emblem_registry_key!("events");

pub struct ExtensionState<'em> {
    lua: Lua,
    phantom: PhantomData<&'em Context<'em>>,
}

impl<'em> ExtensionState<'em> {
    pub fn new(ctx: &'em Context) -> MLuaResult<Self> {
        let params = ctx.lua_params();
        let sandbox_level = params.sandbox_level();

        let lua = if sandbox_level <= SandboxLevel::Unrestricted {
            unsafe { Lua::unsafe_new() }
        } else {
            Lua::new()
        };

        lua.set_app_data(ExtensionData::new());

        preload_sandboxing::restrict_preload(&lua, sandbox_level)?;
        env_extras::import_extras(&lua)?;
        global_sandboxing::restrict_globals(&lua, sandbox_level)?;

        Self::insert_safety_hook(&lua, &params)?;
        Self::setup_event_listeners(&lua)?;

        // TODO(kcza): set args

        lua.load(STD).exec()?;

        Ok(ExtensionState {
            lua,
            phantom: PhantomData,
        })
    }

    fn insert_safety_hook(lua: &Lua, params: &LuaParameters) -> MLuaResult<()> {
        const INSTRUCTION_INTERVAL: u32 = 64;

        let max_mem = params.max_mem();
        let max_steps = params.max_steps();

        lua.set_hook(
            HookTriggers::every_nth_instruction(INSTRUCTION_INTERVAL),
            move |lua, _debug| {
                if let ResourceLimit::Limited(max_mem) = max_mem {
                    if lua.used_memory() >= max_mem {
                        return Err(MLuaError::SafetyError("too much memory used".into()));
                    }
                }

                let mut data: RefMut<'_, ExtensionData> = lua.app_data_mut().unwrap();
                data.curr_step += INSTRUCTION_INTERVAL;
                if let ResourceLimit::Limited(max_steps) = max_steps {
                    if data.curr_step > max_steps {
                        return Err(MLuaError::SafetyError("too many steps".into()));
                    }
                }

                Ok(())
            },
        )
    }

    fn setup_event_listeners(lua: &Lua) -> MLuaResult<()> {
        lua.set_named_registry_value(EVENT_LISTENERS_RKEY, {
            let types = EventType::types();
            let listeners = lua.create_table_with_capacity(
                0,
                types
                    .len()
                    .try_into()
                    .expect("internal error: too many event types"),
            )?;
            for r#type in types {
                listeners.set(r#type.name(), lua.create_table()?)?;
            }
            listeners
        })
    }
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    pub fn add_listener(&self, event: EventType, listener: Value) -> MLuaResult<()> {
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
        let event_listeners = match listeners.get::<_, Option<Table>>(event.r#type().name())? {
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
            Event::IterStart { iter }
            | Event::IterEnd { iter }
            | Event::Done { final_iter: iter } => {
                let event = self.lua.create_table_with_capacity(0, 1)?;
                event.set("iter", iter)?;
                event
            }
        };
        Ok(Value::Table(data))
    }

    pub(crate) fn reiter_requested(&self) -> bool {
        self.lua
            .app_data_ref::<ExtensionData>()
            .expect("internal error: lua app data not set")
            .reiter_requested
    }

    pub(crate) fn reset_reiter_request(&self) {
        self.lua
            .app_data_mut::<ExtensionData>()
            .expect("internal error: lua app data not set")
            .reset_reiter_request();
    }
}

#[cfg(test)]
impl ExtensionState<'_> {
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

pub(crate) struct ExtensionData {
    curr_step: u32,
    reiter_requested: bool,
}

impl Default for ExtensionData {
    fn default() -> Self {
        Self {
            curr_step: 0,
            reiter_requested: false,
        }
    }
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
    IterStart { iter: u32 },
    IterEnd { iter: u32 },
    Done { final_iter: u32 },
}

impl Event {
    pub fn r#type(&self) -> EventType {
        match self {
            Self::IterStart { .. } => EventType::IterStart,

            Self::IterEnd { .. } => EventType::IterEnd,
            Self::Done { .. } => EventType::Done,
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.r#type().fmt(f)
    }
}

#[derive(Copy, Clone)]
pub enum EventType {
    IterStart,
    IterEnd,
    Done,
}

impl EventType {
    pub fn name(&self) -> &str {
        match self {
            Self::IterStart => "iter-start",
            Self::IterEnd => "iter-end",
            Self::Done => "done",
        }
    }

    pub(crate) fn types() -> &'static [EventType] {
        &[Self::IterStart, Self::IterEnd, Self::Done]
    }
}

impl Display for EventType {
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
            let ctx = {
                let mut ctx = Context::test_new();
                ctx.lua_params_mut().set_sandbox_level(level);
                ctx
            };
            let ext_state = ctx.extension_state().unwrap();
            yuescript::Tester::new().test(ext_state.lua());
        }
    }

    // TODO(kcza): test sandboxing application
    // TODO(kcza): test step limits
    // TODO(kcza): test memory limits
}
