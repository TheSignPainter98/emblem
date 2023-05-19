use std::error::Error;

use crate::{
    ast::parsed::ParsedFile,
    build::typesetter::doc::Doc,
    extensions::{Event, ExtensionState},
};

pub(crate) mod doc;

// TODO(kcza): typesettable file -> [fragment]

pub struct Typesetter<'em> {
    max_iters: Option<u32>,
    ext_state: &'em mut ExtensionState,
    #[allow(unused)]
    root: Doc<'em>,
}

impl<'em> Typesetter<'em> {
    pub fn new(ext_state: &'em mut ExtensionState, root: ParsedFile<'em>) -> Self {
        Self {
            max_iters: None,
            ext_state,
            root: Doc::from(root),
        }
    }

    pub fn set_max_iters(mut self, max_iters: u32) -> Self {
        self.max_iters = Some(max_iters);
        self
    }

    pub fn typeset(mut self) -> Result<(), Box<dyn Error>> {
        loop {
            self.ext_state.increment_iter_count();

            self.iter()?;

            if !self.will_reiter() {
                break;
            }
            self.reset_reiter_request();
        }

        self.ext_state.handle(Event::Done)?;

        Ok(())
    }

    fn will_reiter(&self) -> bool {
        self.ext_state.reiter_requested()
            && self.ext_state.curr_iter() < self.max_iters.unwrap_or(u32::MAX)
    }

    fn reset_reiter_request(&self) {
        self.ext_state.reset_reiter_request();
    }

    fn iter(&mut self) -> Result<(), Box<dyn Error>> {
        println!(
            "Doing iteration {} of {}",
            self.ext_state.curr_iter(),
            self.max_iters.unwrap_or(u32::MAX)
        );

        self.ext_state.handle(Event::IterStart)?;
        // TODO(kzca): Evaluate the root.
        self.ext_state.handle(Event::IterEnd)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{extensions::ExtensionData, parser, ExtensionStateBuilder};
    use mlua::{Integer, MetaMethod, Table, ToLua, UserData, Value};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn iter_events() -> Result<(), Box<dyn Error>> {
        let iter_start_indices = Rc::new(RefCell::new(Vec::new()));
        let iter_start_indices_clone = iter_start_indices.clone();
        let iter_end_indices = Rc::new(RefCell::new(Vec::new()));
        let iter_end_indices_clone = iter_end_indices.clone();
        let done_triggered = Rc::new(RefCell::new(false));
        let done_triggered_clone = done_triggered.clone();

        let mut ext_state = ExtensionStateBuilder::default().build().unwrap();
        ext_state.add_listener(
            Event::IterStart,
            Value::Function(ext_state.lua().create_function(move |_, event: Table| {
                let n: Integer = event.get("iter")?;
                iter_start_indices_clone.try_borrow_mut().unwrap().push(n);
                Ok(Value::Nil)
            })?),
        )?;
        ext_state.add_listener(
            Event::IterEnd,
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
            Event::Done,
            Value::Function(ext_state.lua().create_function(move |_, _event: Table| {
                *done_triggered_clone.try_borrow_mut().unwrap() = true;
                Ok(Value::Nil)
            })?),
        )?;

        let typesetter =
            Typesetter::new(&mut ext_state, parser::parse("iter_events.em", "")?).set_max_iters(7);
        typesetter.typeset()?;

        assert_eq!(iter_start_indices.borrow().clone(), [1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(iter_end_indices.borrow().clone(), [1, 2, 3, 4, 5, 6, 7]);
        assert!(*done_triggered.borrow(), "done event was not triggered");

        Ok(())
    }

    #[test]
    fn reiter_request() -> Result<(), Box<dyn Error>> {
        let iter_start_indices = Rc::new(RefCell::new(Vec::new()));
        let iter_start_indices_clone = iter_start_indices.clone();
        let iter_end_indices = Rc::new(RefCell::new(Vec::new()));
        let iter_end_indices_clone = iter_end_indices.clone();
        let done_triggered = Rc::new(RefCell::new(false));
        let done_triggered_clone = done_triggered.clone();

        let mut ext_state = ExtensionStateBuilder::default().build().unwrap();
        ext_state.add_listener(
            Event::IterStart,
            Value::Function(ext_state.lua().create_function(move |_, event: Table| {
                let n: Integer = event.get("iter")?;
                iter_start_indices_clone.try_borrow_mut().unwrap().push(n);
                Ok(Value::Nil)
            })?),
        )?;
        ext_state.add_listener(
            Event::IterEnd,
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
            Event::Done,
            Value::Function(ext_state.lua().create_function(move |_, _event: Table| {
                *done_triggered_clone.try_borrow_mut().unwrap() = true;
                Ok(Value::Nil)
            })?),
        )?;

        Typesetter::new(&mut ext_state, parser::parse("iter_events.em", "")?).typeset()?;

        assert_eq!(
            iter_start_indices.borrow().clone(),
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        );
        assert_eq!(
            iter_end_indices.borrow().clone(),
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        );
        assert!(*done_triggered.borrow(), "done event was not triggered");

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

        let mut ext_state = ExtensionStateBuilder::default().build().unwrap();

        let iter_start_func_called = Rc::new(RefCell::new(false));
        let iter_start_table_called = Rc::new(RefCell::new(false));
        let iter_start_userdata_called = Rc::new(RefCell::new(false));
        {
            let iter_start_func_called_clone = iter_start_func_called.clone();
            let iter_start_table_called_clone = iter_start_table_called.clone();

            ext_state.add_listener(
                Event::IterStart,
                Value::Function(ext_state.lua().create_function(move |_, ()| {
                    *iter_start_func_called_clone.try_borrow_mut().unwrap() = true;
                    Ok(Value::Nil)
                })?),
            )?;
            ext_state.add_listener(
                Event::IterStart,
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
                Event::IterStart,
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
                Event::IterEnd,
                Value::Function(ext_state.lua().create_function(move |_, ()| {
                    *iter_end_func_called_clone.try_borrow_mut().unwrap() = true;
                    Ok(Value::Nil)
                })?),
            )?;
            ext_state.add_listener(
                Event::IterEnd,
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
                Event::IterEnd,
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
                Event::IterEnd,
                Value::Function(ext_state.lua().create_function(move |_, ()| {
                    *done_func_called_clone.try_borrow_mut().unwrap() = true;
                    Ok(Value::Nil)
                })?),
            )?;
            ext_state.add_listener(
                Event::IterEnd,
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
                Event::IterEnd,
                Callable {
                    called: done_userdata_called.clone(),
                }
                .to_lua(ext_state.lua())?,
            )?;
        }

        Typesetter::new(&mut ext_state, parser::parse("event-listeners.em", "")?).typeset()?;

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

        let ext_state = ExtensionStateBuilder::default().build().unwrap();

        for event in [Event::IterStart, Event::IterEnd, Event::Done] {
            assert_eq!(format!("runtime error: non-callable listener integer found when handling {event} event"), ext_state.add_listener(event, Value::Integer(100)).unwrap_err().to_string());

            assert_eq!(
                format!(
                    "runtime error: non-callable listener table found when handling {event} event"
                ),
                ext_state
                    .add_listener(event, Value::Table(ext_state.lua().create_table()?))
                    .unwrap_err()
                    .to_string()
            );

            assert_eq!(format!("runtime error: non-callable listener userdata found when handling {event} event"), ext_state.add_listener(event, NonCallable {}.to_lua(ext_state.lua())?).unwrap_err().to_string());
        }

        // TODO(kcza): valid then revoke!

        Ok(())
    }
}
// TODO(kcza): test events and listeners (functions, callable tables and non-callables)
