mod em;
mod env_extras;
mod global_sandboxing;
mod preload_decls;
mod preload_sandboxing;

use crate::{
    context::{Iteration, LuaParameters, Memory, ResourceLimit, SandboxLevel, Step},
    Context, Error, Result,
};
use em::Em;
use kinded::Kinded;
use mlua::{Error as MLuaError, HookTriggers, Lua, MetaMethod, Table, TableExt, Value};
use std::{cell::RefMut, fmt::Display};
use yuescript::include_yuescript;

#[cfg(test)]
use mlua::AsChunk;

macro_rules! emblem_registry_key {
    ($name:literal) => {
        concat!("__emblem_", $name)
    };
}

static STD: &[u8] = include_yuescript!(cfg!(test), concat!(env!("OUT_DIR"), "/yue"), "std");
const EVENT_LISTENERS_RKEY: &str = emblem_registry_key!("events");

pub struct ExtensionState {
    lua: Lua,
}

impl ExtensionState {
    pub fn new(ctx: &Context) -> Result<Self> {
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

        Self::insert_safety_hook(&lua, params)?;
        Self::setup_event_listeners(&lua)?;

        lua.globals().set("em", Em::new())?;
        // TODO(kcza): set args

        lua.load(STD).exec()?;

        Ok(ExtensionState { lua })
    }

    fn insert_safety_hook(lua: &Lua, params: &LuaParameters) -> Result<()> {
        const INSTRUCTION_INTERVAL: u32 = 1;

        let max_mem = params.max_mem();
        let max_steps = params.max_steps();

        Ok(lua.set_hook(
            HookTriggers::every_nth_instruction(INSTRUCTION_INTERVAL),
            move |lua, _debug| {
                if let ResourceLimit::Limited(max_mem) = max_mem {
                    if Memory(lua.used_memory()) >= max_mem {
                        return Err(MLuaError::SafetyError("too much memory used".into()));
                    }
                }

                let mut data: RefMut<'_, ExtensionData> = lua
                    .app_data_mut()
                    .expect("internal error: expected lua app data to be set");
                data.curr_step += Step(INSTRUCTION_INTERVAL);
                if let ResourceLimit::Limited(max_steps) = max_steps {
                    if data.curr_step > max_steps {
                        return Err(MLuaError::SafetyError("too many steps".into()));
                    }
                }

                Ok(())
            },
        )?)
    }

    fn setup_event_listeners(lua: &Lua) -> Result<()> {
        Ok(lua.set_named_registry_value(EVENT_LISTENERS_RKEY, {
            let event_kinds = EventKind::all();
            let listeners = lua.create_table_with_capacity(
                0,
                event_kinds
                    .len()
                    .try_into()
                    .expect("internal error: too many event types"),
            )?;
            for kind in event_kinds {
                listeners.set(kind.name(), lua.create_table()?)?;
            }
            listeners
        })?)
    }
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    pub fn add_listener(&self, event_type: EventKind, listener: Value) -> Result<()> {
        if !callable(&listener) {
            return Err(Error::uncallable_listener(listener.type_name()));
        }

        let listeners: Table = self.lua.named_registry_value(EVENT_LISTENERS_RKEY)?;
        let Some(event_listeners) = listeners.get::<_, Option<Table>>(event_type.name())? else {
            panic!("internal error: {event_type} event has no listener table")
        };
        Ok(event_listeners.push(listener)?)
    }

    pub fn handle(&self, event: Event) -> Result<()> {
        let listeners: Table = self.lua.named_registry_value(EVENT_LISTENERS_RKEY)?;
        let event_listeners = match listeners.get::<_, Option<Table>>(event.kind().name())? {
            Some(ls) => ls,
            None => panic!("internal error: {event} event has no listeners table"),
        };
        for listener in event_listeners.sequence_values::<Value>() {
            self.call_listener(listener?, event)?;
        }

        Ok(())
    }

    fn call_listener(&self, listener: Value, event: Event) -> Result<()> {
        if let Value::Function(f) = listener {
            f.call(self.event_data(event)?)?;
        } else {
            self.call_listener_method(listener, event)?;
        }

        Ok(())
    }

    fn call_listener_method(&self, listener: Value, event: Event) -> Result<()> {
        let listener_type = listener.type_name();

        match listener.clone() {
            Value::Function(f) => return Ok(f.call(self.event_data(event)?)?),
            Value::Table(t) => return Ok(t.call(self.event_data(event)?)?),
            Value::UserData(u) => {
                if let Ok(mt) = u.get_metatable() {
                    if let Ok(m) = mt.get::<_, Value>(MetaMethod::Call.name()) {
                        match m {
                            Value::Function(f) => {
                                return Ok(f.call((listener, self.event_data(event)?))?);
                            }
                            Value::Table(t) => {
                                return Ok(t.call((listener, self.event_data(event)?))?);
                            }
                            Value::UserData(_) => return self.call_listener_method(m, event),
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }

        Err(Error::uncallable_listener(listener_type))
    }

    fn event_data(&self, event: Event) -> Result<Value> {
        let data = match event {
            Event::IterStart { iter }
            | Event::IterEnd { iter }
            | Event::Done { final_iter: iter } => {
                let event = self.lua.create_table_with_capacity(0, 1)?;
                let Iteration(iter) = iter;
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
impl ExtensionState {
    pub fn run<'lua, C: AsChunk<'lua> + ?Sized>(&'lua self, chunk: &'lua C) -> Result<()> {
        Ok(self.lua.load(chunk).exec()?)
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

#[derive(Debug, Default)]
pub(crate) struct ExtensionData {
    curr_step: Step,
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

#[derive(Copy, Clone, Kinded)]
pub enum Event {
    IterStart { iter: Iteration },
    IterEnd { iter: Iteration },
    Done { final_iter: Iteration },
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IterStart { iter } | Self::IterEnd { iter } | Self::Done { final_iter: iter } => {
                write!(f, "{}({iter})", self.kind())
            }
        }
    }
}

impl EventKind {
    fn name(&self) -> &'static str {
        match self {
            Self::IterStart => "iter-start",
            Self::IterEnd => "iter-end",
            Self::Done => "done",
        }
    }
}

#[cfg(test)]
mod test {
    use mlua::chunk;

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

    #[test]
    fn sandboxing() {
        let canary = "io.stdout";
        let included = SandboxLevel::input_levels()
            .map(|level| {
                let ctx = {
                    let mut ctx = Context::test_new();
                    ctx.lua_params_mut().set_sandbox_level(level);
                    ctx
                };
                let ext_state = ctx.extension_state().unwrap();
                let lua = ext_state.lua();
                lua.load(&format!("return {canary} == nil"))
                    .call::<_, bool>(())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert!(
            included.contains(&true) && included.contains(&false),
            "sandboxing not applied to {canary}: included = {:?}",
            SandboxLevel::input_levels()
                .zip(included.iter())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn steps_limited() -> Result<()> {
        let threshold = Step(10000);
        for limit in [ResourceLimit::Unlimited, ResourceLimit::Limited(threshold)] {
            let ctx = {
                let mut ctx = Context::test_new();
                ctx.lua_params_mut().set_max_steps(limit);
                ctx
            };
            let ext_state = ctx.extension_state()?;
            let lua = ext_state.lua();
            let result = lua
                .load(chunk! {
                    jit.off();
                    tab = {};
                    for i = 1, $threshold do
                        tab[i] = i
                    end
                    return tab;
                })
                .exec();
            assert!(
                result.is_ok() == (limit == ResourceLimit::Unlimited),
                "unexpected result with limit {limit:?}: {result:?}"
            );
        }

        Ok(())
    }

    #[test]
    fn memory_limited() -> Result<()> {
        let threshold = Memory(500000);
        for limit in [ResourceLimit::Unlimited, ResourceLimit::Limited(threshold)] {
            let ctx = {
                let mut ctx = Context::test_new();
                ctx.lua_params_mut().set_max_mem(limit);
                ctx
            };
            let ext_state = ctx.extension_state()?;
            let lua = ext_state.lua();
            let iters = {
                let used_memory = Memory(lua.used_memory());
                assert!(
                    threshold > used_memory,
                    "test invalidated: need threshold > used_memory ({threshold} > {used_memory})"
                );

                let Memory(threshold) = threshold;
                let Memory(used_memory) = used_memory;
                Memory(threshold - used_memory)
            };
            let result = lua
                .load(chunk! {
                    jit.off();
                    tab = {};
                    for i = 1, $iters do
                        tab[i] = i
                    end
                    return tab;
                })
                .exec();
            assert!(
                result.is_ok() == (limit == ResourceLimit::Unlimited),
                "unexpected result with limit {limit:?}: {result:?}",
            );
        }

        Ok(())
    }
}
