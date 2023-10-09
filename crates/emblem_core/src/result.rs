use std::borrow::Cow;

use sealed::sealed;

use crate::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[sealed]
pub trait ErrorContext {
    fn context(self, context: impl Into<Cow<'static, str>>) -> Self;

    fn with_context<C>(self, context_fn: impl Fn() -> C) -> Self
    where
        Self: Sized,
        C: Into<Cow<'static, str>>,
    {
        self.context(context_fn())
    }
}

#[sealed]
impl<T> ErrorContext for Result<T> {
    fn context(self, context: impl Into<Cow<'static, str>>) -> Self {
        self.map_err(|err| err.context(context))
    }
}
