//! Connection point from lua-js to rlua itself.
use rlua::prelude::{};
use crate::error::{Result, Error};
use rlua::Lua;
use crate::value::Value;


pub fn do_string(lua: &Lua, value: Value) -> Result<()> {
    // TODO type assertions, or can we assume it's been handled during the bridging?
    let code: String = match value {
        Value::String(s) => Ok(s),
        v @ _ => Err(Error::Internal(format!("do_string: Expected String, receive: {}", v)))
    }?;

    let r = lua.context(|cx| {
        todo!()
        // cx.load()
    });
    todo!()
}