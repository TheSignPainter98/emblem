use mlua::ToLua;
use sealed::sealed;
use std::{
    error::Error,
    fmt::{Debug, Display},
    num::TryFromIntError,
    ops::{Add, AddAssign},
};

#[sealed]
pub trait Resource:
    Copy
    + Clone
    + Debug
    + Ord
    + Default
    + Add<Self>
    + AddAssign<Self>
    + Display
    + TryFrom<u64>
    + Send
    + Sync
    + 'static
{
    fn default_limit() -> Self;
    fn max_value() -> u64;

    fn parse(raw: &str) -> Result<Self, Box<dyn Error>>
    where
        <Self as TryFrom<u64>>::Error: Error + 'static,
    {
        if raw.is_empty() {
            return Err("need amount".into());
        }

        let max = Self::max_value();

        let (raw_amt, unit): (String, String) = raw.chars().partition(|c| c.is_numeric());
        let amt: u64 = raw_amt
            .parse()
            .map_err(|_| format!("resource limit too large, expected at most {max}",))?;

        let multiplier: u64 = {
            match &unit[..] {
                "K" if max >= 1 << 10 => 1 << 10,
                "M" if max >= 1 << 20 => 1 << 20,
                "G" if max >= 1 << 30 => 1 << 30,
                "" => 1,
                _ => {
                    return Err(format!("unrecognised unit: {}", unit).into());
                }
            }
        };

        if let Some(limit) = amt.checked_mul(multiplier) {
            return Ok(limit.try_into()?);
        }

        Err(format!("resource limit too large, expected at most {}", max,).into())
    }
}

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct Step(pub u32);

#[sealed]
impl Resource for Step {
    fn default_limit() -> Self {
        Self(100_000)
    }

    fn max_value() -> u64 {
        u32::MAX as u64
    }
}

impl Add<Self> for Step {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let Self(l) = self;
        let Self(r) = rhs;
        Self(l + r)
    }
}

impl AddAssign<Self> for Step {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(s) = self;
        write!(f, "{s}")
    }
}

impl<'lua> ToLua<'lua> for Step {
    fn to_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
        let Self(s) = self;
        s.to_lua(lua)
    }
}

impl From<u32> for Step {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl TryFrom<u64> for Step {
    type Error = TryFromIntError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct Memory(pub usize);

#[sealed]
impl Resource for Memory {
    fn default_limit() -> Self {
        Self(100_000)
    }

    fn max_value() -> u64 {
        usize::MAX as u64
    }
}

impl Add<Self> for Memory {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let Self(l) = self;
        let Self(r) = rhs;
        Self(l + r)
    }
}

impl AddAssign<Self> for Memory {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Display for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(s) = self;
        write!(f, "{s}")
    }
}

impl<'lua> ToLua<'lua> for Memory {
    fn to_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
        let Self(s) = self;
        s.to_lua(lua)
    }
}

impl TryFrom<u64> for Memory {
    type Error = TryFromIntError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct Iteration(pub u32);

#[sealed]
impl Resource for Iteration {
    fn default_limit() -> Self {
        Self(5)
    }

    fn max_value() -> u64 {
        u32::MAX as u64
    }
}

impl Add<Self> for Iteration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let Self(l) = self;
        let Self(r) = rhs;
        Self(l + r)
    }
}

impl AddAssign<Self> for Iteration {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Display for Iteration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(s) = self;
        write!(f, "{s}")
    }
}

impl<'lua> ToLua<'lua> for Iteration {
    fn to_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
        let Self(s) = self;
        s.to_lua(lua)
    }
}

impl TryFrom<u64> for Iteration {
    type Error = TryFromIntError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}
