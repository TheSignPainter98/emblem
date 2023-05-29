use crate::RawArgs;
use clap::{
    builder::{StringValueParser, TypedValueParser},
    error::{Error as ClapError, ErrorKind as ClapErrorKind},
    CommandFactory,
};
use emblem_core::context::ResourceLimit as EmblemResourceLimit;
use num::{Bounded, FromPrimitive, Integer, ToPrimitive};
use std::{fmt::Display, str::FromStr};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ResourceLimit<T: Bounded + Clone + Copy + Integer> {
    Unlimited,
    Limited(T),
}

impl<T> ResourceLimit<T>
where
    T: Bounded
        + Clone
        + Copy
        + Display
        + FromPrimitive
        + FromStr
        + Integer
        + ToPrimitive
        + Send
        + Sync
        + 'static,
    <T as FromStr>::Err: Display,
{
    pub(crate) fn parser() -> impl TypedValueParser {
        StringValueParser::new().try_map(Self::try_from)
    }
}

impl<T> TryFrom<String> for ResourceLimit<T>
where
    T: Bounded
        + Clone
        + Copy
        + Display
        + FromPrimitive
        + FromStr
        + Integer
        + ToPrimitive
        + Send
        + Sync
        + 'static,
    <T as FromStr>::Err: Display,
{
    type Error = ClapError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::try_from(&raw[..])
    }
}

impl<T> TryFrom<&str> for ResourceLimit<T>
where
    T: Bounded
        + Clone
        + Copy
        + Display
        + FromPrimitive
        + FromStr
        + Integer
        + ToPrimitive
        + Send
        + Sync
        + 'static,
    <T as FromStr>::Err: Display,
{
    type Error = ClapError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        if raw.is_empty() {
            let mut cmd = RawArgs::command();
            return Err(cmd.error(ClapErrorKind::InvalidValue, "need amount"));
        }

        if raw == "unlimited" {
            return Ok(Self::Unlimited);
        }

        let (raw_amt, unit): (String, String) = raw.chars().partition(|c| c.is_numeric());
        let amt: T = raw_amt.parse().map_err(|_| {
            RawArgs::command().error(
                ClapErrorKind::InvalidValue,
                format!(
                    "resource limit too large, expected at most {}",
                    T::max_value()
                ),
            )
        })?;

        let max = T::max_value();
        let multiplier: T = {
            let max = max.to_u64().expect("internal error: max value too large");
            match &unit[..] {
                "K" if max >= 1 << 10 => T::from_u64(1 << 10).unwrap(),
                "M" if max >= 1 << 20 => T::from_u64(1 << 20).unwrap(),
                "G" if max >= 1 << 30 => T::from_u64(1 << 30).unwrap(),
                "" => T::from_u64(1).unwrap(),
                _ => {
                    let mut cmd = RawArgs::command();
                    return Err(cmd.error(
                        ClapErrorKind::InvalidValue,
                        format!("unrecognised unit: {}", unit),
                    ));
                }
            }
        };

        if max / multiplier < amt {
            let mut cmd = RawArgs::command();
            return Err(cmd.error(
                ClapErrorKind::InvalidValue,
                format!(
                    "resource limit too large, expected at most {}",
                    T::max_value()
                ),
            ));
        }

        Ok(Self::Limited(amt * multiplier))
    }
}

impl<T> Display for ResourceLimit<T>
where
    T: Bounded + Clone + Copy + Display + Integer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unlimited => write!(f, "unlimited"),
            Self::Limited(l) => write!(f, "{l}"),
        }
    }
}

impl<T> From<ResourceLimit<T>> for EmblemResourceLimit<T>
where
    T: Bounded + Integer + Clone + Copy,
{
    fn from(limit: ResourceLimit<T>) -> Self {
        match limit {
            ResourceLimit::Limited(n) => Self::Limited(n),
            ResourceLimit::Unlimited => Self::Unlimited,
        }
    }
}
