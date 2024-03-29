use crate::{
    ast::parsed::ParsedFile,
    build::typesetter::doc::Doc,
    context::Iteration,
    extensions::{Event, EventKind, ExtensionState},
    log::Logger,
    Context, ErrorContext, ResourceLimit, Result,
};

pub(crate) mod doc;

// TODO(kcza): typesettable file -> [fragment]

pub struct Typesetter<'ctx, L: Logger> {
    ctx: &'ctx Context<L>,
    curr_iter: Iteration,
    max_iters: ResourceLimit<Iteration>,
}

impl<'ctx, L: Logger> Typesetter<'ctx, L> {
    pub(crate) fn new(ctx: &'ctx Context<L>) -> Self {
        Self {
            ctx,
            curr_iter: Iteration(0),
            max_iters: ctx.typesetter_params().max_iters(),
        }
    }

    pub fn typeset(mut self, root: ParsedFile) -> Result<()> {
        let mut root = Doc::from(root);
        let ext_state = self.ctx.extension_state()?;
        loop {
            self.iter(ext_state, &mut root)?;

            if !self.will_reiter(ext_state) {
                break;
            }
            ext_state.reset_reiter_request();
        }

        ext_state
            .handle(Event::Done {
                final_iter: self.curr_iter,
            })
            .with_context(|| format!("failed to handle {} event", EventKind::Done))?;

        Ok(())
    }

    fn will_reiter(&self, ext_state: &ExtensionState) -> bool {
        ext_state.reiter_requested() && self.max_iters.lt(self.curr_iter)
    }

    fn iter(&mut self, ext_state: &ExtensionState, _root: &mut Doc) -> Result<()> {
        self.curr_iter += Iteration(1);

        let Iteration(iter) = &self.curr_iter;
        println!("Doing iteration {iter} of {:?}", self.max_iters);

        ext_state
            .handle(Event::IterStart {
                iter: self.curr_iter,
            })
            .with_context(|| format!("failed to handle {} event", EventKind::IterStart))?;
        // TODO(kzca): Evaluate the root.
        ext_state
            .handle(Event::IterEnd {
                iter: self.curr_iter,
            })
            .with_context(|| format!("failed to handle {} event", EventKind::IterEnd))?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        extensions::{EventKind, ExtensionData},
        parser,
    };
    use mlua::{Integer, MetaMethod, Table, ToLua, UserData, Value};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn iter_events() -> Result<()> {
        let iter_start_indices = Rc::new(RefCell::new(Vec::new()));
        let iter_start_indices_clone = iter_start_indices.clone();
        let iter_end_indices = Rc::new(RefCell::new(Vec::new()));
        let iter_end_indices_clone = iter_end_indices.clone();
        let done_triggered = Rc::new(RefCell::new(Vec::new()));
        let done_triggered_clone = done_triggered.clone();

        let ctx = {
            let mut ctx = Context::test_new();
            ctx.typesetter_params_mut()
                .set_max_iters(ResourceLimit::Limited(Iteration(7)));
            ctx
        };
        let ext_state = ctx.extension_state()?;
        ext_state.add_listener(
            EventKind::IterStart,
            Value::Function(ext_state.lua().create_function(move |_, event: Table| {
                let n: Integer = event.get("iter")?;
                iter_start_indices_clone.try_borrow_mut().unwrap().push(n);
                Ok(Value::Nil)
            })?),
        )?;
        ext_state.add_listener(
            EventKind::IterEnd,
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
            EventKind::Done,
            Value::Function(ext_state.lua().create_function(move |_, event: Table| {
                let n: Integer = event.get("iter")?;
                done_triggered_clone.try_borrow_mut().unwrap().push(n);
                Ok(Value::Nil)
            })?),
        )?;

        let typesetter = ctx.typesetter();
        typesetter.typeset(
            parser::parse(
                ctx.alloc_file_name("iter_events.em"),
                ctx.alloc_file_content(""),
            )
            .unwrap(),
        )?;

        assert_eq!(iter_start_indices.borrow().clone(), [1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(iter_end_indices.borrow().clone(), [1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(done_triggered.borrow().clone(), [7]);

        Ok(())
    }

    #[test]
    fn reiter_request() -> Result<()> {
        let iter_start_indices = Rc::new(RefCell::new(Vec::new()));
        let iter_start_indices_clone = iter_start_indices.clone();
        let iter_end_indices = Rc::new(RefCell::new(Vec::new()));
        let iter_end_indices_clone = iter_end_indices.clone();
        let done_triggered = Rc::new(RefCell::new(Vec::new()));
        let done_triggered_clone = done_triggered.clone();

        let ctx = Context::test_new();
        let ext_state = ctx.extension_state()?;
        ext_state.add_listener(
            EventKind::IterStart,
            Value::Function(ext_state.lua().create_function(move |_, event: Table| {
                let n: Integer = event.get("iter")?;
                iter_start_indices_clone.try_borrow_mut().unwrap().push(n);
                Ok(Value::Nil)
            })?),
        )?;
        ext_state.add_listener(
            EventKind::IterEnd,
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
            EventKind::Done,
            Value::Function(ext_state.lua().create_function(move |_, event: Table| {
                let n: Integer = event.get("iter")?;
                done_triggered_clone.try_borrow_mut().unwrap().push(n);
                Ok(Value::Nil)
            })?),
        )?;

        Typesetter::new(&ctx).typeset(
            parser::parse(
                ctx.alloc_file_name("iter_events.em"),
                ctx.alloc_file_content(""),
            )
            .unwrap(),
        )?;

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
    fn event_listeners() -> Result<()> {
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
        let ext_state = ctx.extension_state()?;

        let iter_start_func_called = Rc::new(RefCell::new(false));
        let iter_start_table_called = Rc::new(RefCell::new(false));
        let iter_start_userdata_called = Rc::new(RefCell::new(false));
        {
            let iter_start_func_called_clone = iter_start_func_called.clone();
            let iter_start_table_called_clone = iter_start_table_called.clone();

            ext_state.add_listener(
                EventKind::IterStart,
                Value::Function(ext_state.lua().create_function(move |_, ()| {
                    *iter_start_func_called_clone.try_borrow_mut().unwrap() = true;
                    Ok(Value::Nil)
                })?),
            )?;
            ext_state.add_listener(
                EventKind::IterStart,
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
                EventKind::IterStart,
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
                EventKind::IterEnd,
                Value::Function(ext_state.lua().create_function(move |_, ()| {
                    *iter_end_func_called_clone.try_borrow_mut().unwrap() = true;
                    Ok(Value::Nil)
                })?),
            )?;
            ext_state.add_listener(
                EventKind::IterEnd,
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
                EventKind::IterEnd,
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
                EventKind::IterEnd,
                Value::Function(ext_state.lua().create_function(move |_, ()| {
                    *done_func_called_clone.try_borrow_mut().unwrap() = true;
                    Ok(Value::Nil)
                })?),
            )?;
            ext_state.add_listener(
                EventKind::IterEnd,
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
                EventKind::IterEnd,
                Callable {
                    called: done_userdata_called.clone(),
                }
                .to_lua(ext_state.lua())?,
            )?;
        }

        Typesetter::new(&ctx).typeset(
            parser::parse(
                ctx.alloc_file_name("event-listeners.em"),
                ctx.alloc_file_content(""),
            )
            .unwrap(),
        )?;

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
    fn invalid_event_listeners() -> Result<()> {
        struct NonCallable {}

        impl UserData for NonCallable {}

        let ctx = Context::test_new();
        let ext_state = ctx.extension_state()?;
        for event_kind in EventKind::all() {
            assert_eq!(
                format!("integer is not callable"),
                ext_state
                    .add_listener(*event_kind, Value::Integer(100))
                    .unwrap_err()
                    .to_string()
            );

            assert_eq!(
                format!("table is not callable"),
                ext_state
                    .add_listener(*event_kind, Value::Table(ext_state.lua().create_table()?))
                    .unwrap_err()
                    .to_string()
            );

            assert_eq!(
                format!("userdata is not callable"),
                ext_state
                    .add_listener(*event_kind, NonCallable {}.to_lua(ext_state.lua())?)
                    .unwrap_err()
                    .to_string()
            );
        }

        Ok(())
    }

    #[test]
    fn invalidated_event_listeners() -> Result<()> {
        for event_kind in EventKind::all() {
            let ctx = Context::test_new();
            let ext_state = ctx.extension_state()?;
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
                    .add_listener(*event_kind, Value::Table(table.clone()))
                    .is_ok());

                table.set_metatable(None);
            }

            let err = Typesetter::new(&ctx)
                .typeset(parser::parse(
                    ctx.alloc_file_name("event-listeners.em"),
                    ctx.alloc_file_content(""),
                )?)
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
