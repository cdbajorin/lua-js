//! Rust intermediate state between JS and Lua Value types.

use crate::error::Error;

use rlua::prelude::{LuaContext, LuaValue};
use rlua::{FromLua, ToLua};

use crate::js_traits::{FromJs, ToJs};
use neon::handle::Handle;
use neon::prelude::Context as JsContext;
use neon::result::{NeonResult, Throw};
use neon::types::{JsArray, JsBoolean, JsNull, JsNumber, JsObject, JsString, JsUndefined, JsValue};
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub enum Value {
    // This lets us choose what our JS output is.
    // Both convert to nil on the Lua side
    Null,
    Undefined,
    Boolean(bool),
    String(String),
    // Lua has int/float. We'll treat both with Number.
    Number(f64),
    Array(Vec<Value>),
    Error(Error),
}

impl<'lua> ToLua<'lua> for Value {
    fn to_lua(self, lua: LuaContext<'lua>) -> rlua::Result<LuaValue<'lua>> {
        match self {
            Value::String(s) => {
                let lua_str = lua.create_string(s.as_bytes())?;
                Ok(LuaValue::String(lua_str))
            }
            Value::Number(f) => Ok(LuaValue::Number(f)),
            Value::Undefined | Value::Null => Ok(LuaValue::Nil),
            k @ _ => unimplemented!("ToLua: non string/number {:?}", k),
        }
    }
}

// TODO from_lua for Value
impl<'lua> FromLua<'lua> for Value {
    fn from_lua(lua_value: LuaValue<'lua>, _lua: LuaContext<'lua>) -> rlua::Result<Self> {
        match lua_value {
            LuaValue::Nil => Ok(Value::Null),
            LuaValue::Boolean(b) => Ok(Value::Boolean(b)),
            LuaValue::Integer(i) => Ok(Value::Number(i as f64)),
            LuaValue::Number(f) => Ok(Value::Number(f)),
            LuaValue::String(s) => {
                let s = s.to_str()?;
                Ok(Value::String(s.to_owned()))
            }
            LuaValue::LightUserData(_) => unimplemented!("LightUserData"),
            LuaValue::Table(_) => unimplemented!("Table"),
            LuaValue::Function(_) => unimplemented!("Function"),
            LuaValue::Thread(_) => unimplemented!("Thread"),
            LuaValue::UserData(_) => unimplemented!("UserData"),
            LuaValue::Error(e) => {
                // TODO what to do with error values instead of calls?
                Ok(Value::Error(Error::Lua(e)))
            }
        }
    }
}

impl ToJs for Value {
    fn to_js<'a, CX: neon::context::Context<'a>>(
        &self,
        cx: &mut CX,
    ) -> neon::result::JsResult<'a, JsValue> {
        match self {
            Value::String(s) => Ok(cx.string(s).upcast()),
            Value::Number(f) => Ok(cx.number(*f).upcast()),
            Value::Error(e) => cx.throw_error(e.to_string()),
            Value::Null => Ok(cx.null().upcast()),
            Value::Undefined => Ok(cx.undefined().upcast()),
            Value::Boolean(b) => Ok(cx.boolean(*b).upcast()),
            Value::Array(_v) => todo!(),
        }
    }
}

impl FromJs for Value {
    fn from_js<'a, CX: JsContext<'a>>(handle: Handle<JsValue>, cx: &mut CX) -> NeonResult<Self> {
        if handle.is_a::<JsNull>() || handle.is_a::<JsUndefined>() {
            Ok(Value::Null)
        } else if handle.is_a::<JsNumber>() {
            let num = handle.downcast_or_throw::<JsNumber, CX>(cx)?.value();
            Ok(Value::Number(num))
        } else if handle.is_a::<JsString>() {
            let s = handle.downcast_or_throw::<JsString, CX>(cx)?.value();
            Ok(Value::String(s))
        } else if handle.is_a::<JsBoolean>() {
            let b = handle.downcast_or_throw::<JsBoolean, CX>(cx)?.value();
            Ok(Value::Boolean(b))
        } else if handle.is_a::<JsArray>() {
            unimplemented!("JsArray to Lua");
        } else if handle.is_a::<JsObject>() {
            unimplemented!("JsObject to Lua");
        } else {
            Err(Throw)
        }
    }
}

// TODO Display for Value
impl std::fmt::Display for Value {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!("Display for Value");
    }
}
