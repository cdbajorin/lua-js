use rlua::prelude::LuaError;
use std::fmt::Formatter;


#[derive(Debug,Clone)]
pub enum Error {
    // We're passing as a string because Send is required by
    // Task, and JsValue doesn't implement it. Is an IR error even necessary?
    // Js(String),
    Lua(LuaError),
    // where exactly can this happen, or is it a fallback?
    // Internal(String)
}

// pub type Result<'a,T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for Error {
    // TODO implement source, or just rely on error enum instead of struct?
}


impl From<LuaError> for Error {
    fn from(err: LuaError) -> Self {
        Error::Lua(err)
    }
}

// TODO figure out how to use errors with Task, so we can pass errors to both
//  callbacks as well as throw
// impl From<JsError> for Error {
//     fn from(err: JsError) -> Self {
//         Error::Js(err.to_string())
//     }
// }
