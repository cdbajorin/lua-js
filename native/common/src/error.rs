use mlua::prelude::LuaError;
use neon::result::Throw;
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub enum Error {
    Js(String),
    Lua(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Lua(e) => write!(f, "{}", e),
            Error::Js(e) => write!(f, "{}", e),
        }
    }
}

impl From<LuaError> for Error {
    fn from(err: LuaError) -> Self {
        Error::Lua(err.to_string())
    }
}

impl From<Throw> for Error {
    fn from(err: Throw) -> Self {
        Error::Js(format!("{}", err))
    }
}
