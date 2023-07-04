use crate::RawArgs;
use clap::{
    builder::{StringValueParser, TypedValueParser},
    error::{Error as ClapError, ErrorKind as ClapErrorKind},
    CommandFactory,
};
use emblem_core::context::{Resource, ResourceLimit as EmblemResourceLimit};
use std::{error::Error, fmt::Display};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ResourceLimit<T: Resource> {
    Unlimited,
    Limited(T),
}

impl<T: Resource> ResourceLimit<T>
where
    <T as TryFrom<u64>>::Error: Error,
{
    pub(crate) fn parser() -> impl TypedValueParser {
        StringValueParser::new().try_map(Self::try_from)
    }
}

impl<T: Resource> TryFrom<String> for ResourceLimit<T>
where
    <T as TryFrom<u64>>::Error: Error,
{
    type Error = ClapError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::try_from(&raw[..])
    }
}

impl<T: Resource> TryFrom<&str> for ResourceLimit<T>
where
    <T as TryFrom<u64>>::Error: Error,
{
    type Error = ClapError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        if raw == "unlimited" {
            return Ok(Self::Unlimited);
        }

        Ok(Self::Limited(T::parse(raw).map_err(|err| {
            RawArgs::command().error(ClapErrorKind::InvalidValue, err.to_string())
        })?))
    }
}

impl<T: Resource> Display for ResourceLimit<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unlimited => write!(f, "unlimited"),
            Self::Limited(l) => write!(f, "{l}"),
        }
    }
}

impl<T: Resource> From<ResourceLimit<T>> for EmblemResourceLimit<T> {
    fn from(limit: ResourceLimit<T>) -> Self {
        match limit {
            ResourceLimit::Limited(n) => Self::Limited(n),
            ResourceLimit::Unlimited => Self::Unlimited,
        }
    }
}

impl<T: Resource> Default for ResourceLimit<T> {
    fn default() -> Self {
        Self::Limited(T::default_limit())
    }
}
