use std::error::Error;

use crate::{
    ast::parsed::ParsedFile,
    build::typesetter::doc::Doc,
    extensions::{Event, ExtensionState},
    Context, ResourceLimit,
};

pub(crate) mod doc;

// TODO(kcza): typesettable file -> [fragment]

pub struct Typesetter<'em> {
    ext_state: &'em mut ExtensionState<'em>,
    curr_iter: u32,
    max_iters: ResourceLimit<u32>,
}

impl<'em> Typesetter<'em> {
    pub fn new(ctx: &'em Context<'em>, ext_state: &'em mut ExtensionState<'em>) -> Self {
        Self {
            ext_state,
            curr_iter: 0,
            max_iters: ctx.typesetter_params().max_iters(),
        }
    }

    pub fn typeset(mut self, root: ParsedFile<'em>) -> Result<(), Box<dyn Error>> {
        let mut root = Doc::from(root);
        loop {
            self.iter(&mut root)?;

            if !self.will_reiter() {
                break;
            }
            self.reset_reiter_request();
        }

        self.ext_state.handle(Event::Done {
            final_iter: self.curr_iter,
        })?;

        Ok(())
    }

    fn will_reiter(&self) -> bool {
        self.ext_state.reiter_requested()
            && self.curr_iter < self.max_iters.limit().unwrap_or(u32::MAX)
    }

    fn reset_reiter_request(&self) {
        self.ext_state.reset_reiter_request();
    }

    fn iter(&mut self, _root: &mut Doc<'em>) -> Result<(), Box<dyn Error>> {
        self.curr_iter += 1;

        println!("Doing iteration {} of {:?}", self.curr_iter, self.max_iters);

        self.ext_state.handle(Event::IterStart {
            iter: self.curr_iter,
        })?;
        // TODO(kzca): Evaluate the root.
        self.ext_state.handle(Event::IterEnd {
            iter: self.curr_iter,
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        extensions::{EventType, ExtensionData},
        parser,
    };
    use mlua::{Integer, MetaMethod, Table, ToLua, UserData, Value};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn iter_events() -> Result<(), Box<dyn Error>> {
        let iter_start_indices = Rc::new(RefCell::new(Vec::new()));
        let iter_start_indices_clone = iter_start_indices.clone();
        let iter_end_indices = Rc::new(RefCell::new(Vec::new()));
        let iter_end_indices_clone = iter_end_indices.clone();
        let done_triggered = Rc::new(RefCell::new(Vec::new()));
        let done_triggered_clone = done_triggered.clone();

        let ctx = {
            let mut ctx = Context::test_new();
            ctx.typesetter_params_mut()
                .set_max_iters(ResourceLimit::Limited(7));
            ctx
        };
        let mut ext_state = ctx.extension_state()?;
        ext_state.add_listener(
            EventType::IterStart,
            Value::Function(ext_state.lua().create_function(move |_, event: Table| {
                let n: Integer = event.get("iter")?;
                iter_start_indices_clone.try_borrow_mut().unwrap().push(n);
                Ok(Value::Nil)
            })?),
        )?;
        ext_state.add_listener(
            EventType::IterEnd,
            Value::Function(ext_state.lua().create_function(move |lua, event: Table| {
                let n: Integer = event.get("iter")?;
                iter_end_indices_clone.try_borrow_mut().unwrap().push(n);

                lua.app_data_mut::<ExtensionData>()
                    .unwrap()
                    .request_reiter();

                Ok(Value::Nil)
            })?),
        )?;
        ext_state.add_listener(
            EventType::Done,
            Value::Function(ext_state.lua().create_function(move |_, event: Table| {
                let n: Integer = event.get("iter")?;
                done_triggered_clone.try_borrow_mut().unwrap().push(n);
                Ok(Value::Nil)
            })?),
        )?;

        let typesetter = Typesetter::new(&ctx, &mut ext_state);
        typesetter.typeset(parser::parse("iter_events.em", "")?)?;

        assert_eq!(iter_start_indices.borrow().clone(), [1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(iter_end_indices.borrow().clone(), [1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(done_triggered.borrow().clone(), [7]);

        Ok(())
    }

    #[test]
    fn reiter_request() -> Result<(), Box<dyn Error>> {
        let iter_start_indices = Rc::new(RefCell::new(Vec::new()));
        let iter_start_indices_clone = iter_start_indices.clone();
        let iter_end_indices = Rc::new(RefCell::new(Vec::new()));
        let iter_end_indices_clone = iter_end_indices.clone();
        let done_triggered = Rc::new(RefCell::new(Vec::new()));
        let done_triggered_clone = done_triggered.clone();

        let ctx = Context::test_new();
        let mut ext_state = ctx.extension_state()?;
        ext_state.add_listener(
            EventType::IterStart,
            Value::Function(ext_state.lua().create_function(move |_, event: Table| {
                let n: Integer = event.get("iter")?;
                iter_start_indices_clone.try_borrow_mut().unwrap().push(n);
                Ok(Value::Nil)
            })?),
        )?;
        ext_state.add_listener(
            EventType::IterEnd,
            Value::Function(ext_state.lua().create_function(move |lua, event: Table| {
                let n: Integer = event.get("iter")?;
                iter_end_indices_clone.try_borrow_mut().unwrap().push(n);

                if n < 10 {
                    lua.app_data_mut::<ExtensionData>()
                        .unwrap()
                        .request_reiter();
                }

                Ok(Value::Nil)
            })?),
        )?;
        ext_state.add_listener(
            EventType::Done,
            Value::Function(ext_state.lua().create_function(move |_, event: Table| {
                let n: Integer = event.get("iter")?;
                done_triggered_clone.try_borrow_mut().unwrap().push(n);
                Ok(Value::Nil)
            })?),
        )?;

        Typesetter::new(&ctx, &mut ext_state).typeset(parser::parse("iter_events.em", "")?)?;

        assert_eq!(
            iter_start_indices.borrow().clone(),
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        );
        assert_eq!(
            iter_end_indices.borrow().clone(),
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        );
        assert_eq!(done_triggered.borrow().clone(), [10]);

        Ok(())
    }

    #[test]
    fn event_listeners() -> Result<(), Box<dyn Error>> {
        struct Callable {
            called: Rc<RefCell<bool>>,
        }

        impl UserData for Callable {
            fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
                methods.add_meta_method(MetaMethod::Call, |_, this, ()| {
                    *this.called.try_borrow_mut().unwrap() = true;
                    Ok(Value::Nil)
                });
            }
        }

        let ctx = Context::test_new();
        let mut ext_state = ctx.extension_state()?;

        let iter_start_func_called = Rc::new(RefCell::new(false));
        let iter_start_table_called = Rc::new(RefCell::new(false));
        let iter_start_userdata_called = Rc::new(RefCell::new(false));
        {
            let iter_start_func_called_clone = iter_start_func_called.clone();
            let iter_start_table_called_clone = iter_start_table_called.clone();

            ext_state.add_listener(
                EventType::IterStart,
                Value::Function(ext_state.lua().create_function(move |_, ()| {
                    *iter_start_func_called_clone.try_borrow_mut().unwrap() = true;
                    Ok(Value::Nil)
                })?),
            )?;
            ext_state.add_listener(
                EventType::IterStart,
                Value::Table({
                    let table = ext_state.lua().create_table()?;
                    table.set_metatable(Some({
                        let mt = ext_state.lua().create_table()?;
                        mt.set(
                            MetaMethod::Call.name(),
                            Value::Function(ext_state.lua().create_function(
                                move |_, _: Table| {
                                    *iter_start_table_called_clone.try_borrow_mut().unwrap() = true;
                                    Ok(Value::Nil)
                                },
                            )?),
                        )?;
                        mt
                    }));
                    table
                }),
            )?;
            ext_state.add_listener(
                EventType::IterStart,
                Callable {
                    called: iter_start_userdata_called.clone(),
                }
                .to_lua(ext_state.lua())?,
            )?;
        }

        let iter_end_func_called = Rc::new(RefCell::new(false));
        let iter_end_table_called = Rc::new(RefCell::new(false));
        let iter_end_userdata_called = Rc::new(RefCell::new(false));
        {
            let iter_end_func_called_clone = iter_end_func_called.clone();
            let iter_end_table_called_clone = iter_end_table_called.clone();

            ext_state.add_listener(
                EventType::IterEnd,
                Value::Function(ext_state.lua().create_function(move |_, ()| {
                    *iter_end_func_called_clone.try_borrow_mut().unwrap() = true;
                    Ok(Value::Nil)
                })?),
            )?;
            ext_state.add_listener(
                EventType::IterEnd,
                Value::Table({
                    let table = ext_state.lua().create_table()?;
                    table.set_metatable(Some({
                        let mt = ext_state.lua().create_table()?;
                        mt.set(
                            MetaMethod::Call.name(),
                            Value::Function(ext_state.lua().create_function(
                                move |_, _: Table| {
                                    *iter_end_table_called_clone.try_borrow_mut().unwrap() = true;
                                    Ok(Value::Nil)
                                },
                            )?),
                        )?;
                        mt
                    }));
                    table
                }),
            )?;
            ext_state.add_listener(
                EventType::IterEnd,
                Callable {
                    called: iter_end_userdata_called.clone(),
                }
                .to_lua(ext_state.lua())?,
            )?;
        }

        let done_func_called = Rc::new(RefCell::new(false));
        let done_table_called = Rc::new(RefCell::new(false));
        let done_userdata_called = Rc::new(RefCell::new(false));
        {
            let done_func_called_clone = done_func_called.clone();
            let done_table_called_clone = done_table_called.clone();

            ext_state.add_listener(
                EventType::IterEnd,
                Value::Function(ext_state.lua().create_function(move |_, ()| {
                    *done_func_called_clone.try_borrow_mut().unwrap() = true;
                    Ok(Value::Nil)
                })?),
            )?;
            ext_state.add_listener(
                EventType::IterEnd,
                Value::Table({
                    let table = ext_state.lua().create_table()?;
                    table.set_metatable(Some({
                        let mt = ext_state.lua().create_table()?;
                        mt.set(
                            MetaMethod::Call.name(),
                            Value::Function(ext_state.lua().create_function(
                                move |_, _: Table| {
                                    *done_table_called_clone.try_borrow_mut().unwrap() = true;
                                    Ok(Value::Nil)
                                },
                            )?),
                        )?;
                        mt
                    }));
                    table
                }),
            )?;
            ext_state.add_listener(
                EventType::IterEnd,
                Callable {
                    called: done_userdata_called.clone(),
                }
                .to_lua(ext_state.lua())?,
            )?;
        }

        Typesetter::new(&ctx, &mut ext_state).typeset(parser::parse("event-listeners.em", "")?)?;

        assert!(*iter_start_func_called.borrow());
        assert!(*iter_start_table_called.borrow());
        assert!(*iter_start_userdata_called.borrow());
        assert!(*iter_end_func_called.borrow());
        assert!(*iter_end_table_called.borrow());
        assert!(*iter_end_userdata_called.borrow());
        assert!(*done_func_called.borrow());
        assert!(*done_table_called.borrow());
        assert!(*done_userdata_called.borrow());

        Ok(())
    }

    #[test]
    fn invalid_event_listeners() -> Result<(), Box<dyn Error>> {
        struct NonCallable {}

        impl UserData for NonCallable {}

        let ctx = Context::test_new();
        let ext_state = ctx.extension_state()?;
        for event_type in EventType::types() {
            assert_eq!(format!("runtime error: non-callable listener integer found when handling {event_type} event"), ext_state.add_listener(*event_type, Value::Integer(100)).unwrap_err().to_string());

            assert_eq!(
                format!(
                    "runtime error: non-callable listener table found when handling {event_type} event"
                ),
                ext_state
                    .add_listener(*event_type, Value::Table(ext_state.lua().create_table()?))
                    .unwrap_err()
                    .to_string()
            );

            assert_eq!(format!("runtime error: non-callable listener userdata found when handling {event_type} event"), ext_state.add_listener(*event_type, NonCallable {}.to_lua(ext_state.lua())?).unwrap_err().to_string());
        }

        Ok(())
    }

    #[test]
    fn invalidated_event_listeners() -> Result<(), Box<dyn Error>> {
        for event_type in EventType::types() {
            let ctx = Context::test_new();
            let mut ext_state = ctx.extension_state()?;
            let handler_called = Rc::new(RefCell::new(false));

            {
                let table = {
                    let table = ext_state.lua().create_table()?;
                    table.set_metatable(Some({
                        let mt = ext_state.lua().create_table()?;
                        let handler_called = handler_called.clone();
                        mt.set(
                            MetaMethod::Call.name(),
                            Value::Function(ext_state.lua().create_function(
                                move |_, _: Table| {
                                    *handler_called.try_borrow_mut().unwrap() = true;
                                    Ok(Value::Nil)
                                },
                            )?),
                        )?;
                        mt
                    }));
                    table
                };
                assert!(ext_state
                    .add_listener(*event_type, Value::Table(table.clone()))
                    .is_ok());

                table.set_metatable(None);
            }

            let err = Typesetter::new(&ctx, &mut ext_state)
                .typeset(parser::parse("event-listeners.em", "")?)
                .unwrap_err();
            assert!(
                err.to_string()
                    .contains("runtime error: attempt to call a table value"),
                "unexpected error: {err}"
            );

            assert!(!*handler_called.borrow(), "handler unexpectedly called");
        }

        Ok(())
    }
}
