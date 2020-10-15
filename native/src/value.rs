//! Rust intermediate state between JS and Lua Value types.

use crate::error::Error;

use rlua::prelude::{LuaContext, LuaValue};
use rlua::{FromLua, ToLua};

use crate::js_traits::{FromJs, ToJs};
use neon::handle::Handle;
use neon::prelude::*;
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
    // k/v pairs, indexed values.
    ObjectLike(Vec<(Value, Value)>, Vec<Value>),
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
            Value::Boolean(b) => Ok(LuaValue::Boolean(b)),
            Value::Array(values) => {
                let table = lua.create_table()?;
                for (i, v) in values.into_iter().enumerate() {
                    table.raw_set(i, v)?;
                }
                Ok(LuaValue::Table(table))
            }
            Value::Error(_err) => unimplemented!("ToLua for Error"),
            Value::ObjectLike(_kv_pairs, _array_like) => {
                unimplemented!("ToLua for ObjectLike");
            }
        }
    }
}

impl<'lua> FromLua<'lua> for Value {
    fn from_lua(lua_value: LuaValue<'lua>, lua: LuaContext<'lua>) -> rlua::Result<Self> {
        match lua_value {
            LuaValue::Nil => Ok(Value::Null),
            LuaValue::Boolean(b) => Ok(Value::Boolean(b)),
            LuaValue::Integer(i) => Ok(Value::Number(i as f64)),
            LuaValue::Number(f) => Ok(Value::Number(f)),
            LuaValue::String(s) => {
                let s = s.to_str()?;
                Ok(Value::String(s.to_owned()))
            }
            LuaValue::Table(table) => {
                // We're simulating an object/array similar to how Lua implements it.
                // At the moment, even though Lua states iteration order isn't guaranteed,
                // I haven't bumped into a situation yet where it's an issue.
                let mut kv_pairs: Vec<(Value, Value)> = vec![];
                let len = table.len()?;
                let mut indexed_values: Vec<Value> = Vec::with_capacity(len as usize);

                for pair in table.pairs() {
                    let (key, value) = pair?;
                    let value_v = Value::from_lua(value, lua)?;
                    match key {
                        LuaValue::Integer(n) => {
                            let k_n = (n - 1) as usize;
                            // Sparse-ish vec representing the array-like segment.
                            // If we haven't filled it yet, we're filling with nulls.
                            // Not sure if there's a better way to implement this?
                            while indexed_values.len() < k_n {
                                indexed_values.push(Value::Undefined);
                            }
                            indexed_values.insert(k_n, value_v);
                        }
                        LuaValue::Number(f) => {
                            // floats are converted to strings, as that's the only representation
                            // that makes any sense on the JS side.
                            let float_key = f.to_string();
                            kv_pairs.push((Value::String(float_key), value_v));
                        }
                        LuaValue::String(s) => {
                            let string = s.to_str()?.to_owned();
                            kv_pairs.push((Value::String(string), value_v));
                        }
                        // Of course, there is no display for these...
                        LuaValue::Table(_) => panic!("Cannot convert Lua `table` to JS object key"),
                        LuaValue::Function(_) => {
                            panic!("Cannot convert Lua `function` to JS object key")
                        }
                        LuaValue::Thread(_) => {
                            panic!("Cannot convert Lua `thread` to JS object key")
                        }
                        LuaValue::Nil => panic!("Cannot convert Lua `nil` to JS object key"),
                        LuaValue::Boolean(_) => {
                            panic!("Cannot convert Lua `boolean` to JS object key")
                        }
                        LuaValue::LightUserData(_) => {
                            panic!("Cannot convert Lua `userData` to JS object key")
                        }
                        LuaValue::UserData(_) => {
                            panic!("Cannot convert Lua `userData` to JS object key")
                        }
                        LuaValue::Error(_) => panic!("Cannot convert Lua `error` to JS object key"),
                    };
                }
                Ok(Value::ObjectLike(kv_pairs, indexed_values))
            }
            LuaValue::Function(_) => unimplemented!("Function"),
            LuaValue::Thread(_) => unimplemented!("Thread"),
            LuaValue::UserData(_) => unimplemented!("UserData"),
            LuaValue::LightUserData(_) => unimplemented!("LightUserData"),
            LuaValue::Error(e) => {
                // TODO what to do with error values instead of calls?
                Ok(Value::Error(Error::Lua(e)))
            }
        }
    }
}

impl ToJs for Value {
    fn to_js<'a, CX: Context<'a>>(&self, cx: &mut CX) -> neon::result::JsResult<'a, JsValue> {
        match self {
            Value::String(s) => Ok(cx.string(s).upcast()),
            Value::Number(f) => Ok(cx.number(*f).upcast()),
            Value::Error(e) => cx.throw_error(e.to_string()),
            Value::Null => Ok(cx.null().upcast()),
            Value::Undefined => Ok(cx.undefined().upcast()),
            Value::Boolean(b) => Ok(cx.boolean(*b).upcast()),
            Value::Array(values) => {
                let array = cx.empty_array();
                for (i, v) in values.into_iter().enumerate() {
                    let value_handle = v.to_js(cx)?;
                    array.set(cx, i as u32, value_handle)?;
                }
                Ok(array.upcast())
            }
            Value::ObjectLike(pairs, array_like) => {
                // let obj = cx.empty_object();
                let obj: Handle<JsObject> = if array_like.len() > 0 {
                    cx.empty_array().downcast_or_throw::<JsObject, CX>(cx)?
                } else {
                    cx.empty_object().downcast_or_throw::<JsObject, CX>(cx)?
                };

                for (idx, val) in array_like.into_iter().enumerate() {
                    let js_val = val.to_js(cx)?;
                    obj.set(cx, idx as u32, js_val)?;
                }

                for (key, value) in pairs {
                    let js_key = key.to_js(cx)?;
                    let js_value = value.to_js(cx)?;
                    obj.set(cx, js_key, js_value)?;
                }
                Ok(obj.upcast())
            }
        }
    }
}

impl FromJs for Value {
    fn from_js<'a, CX: Context<'a>>(handle: Handle<JsValue>, cx: &mut CX) -> NeonResult<Self> {
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
            let arr = handle.downcast_or_throw::<JsArray, CX>(cx)?;
            let values: Vec<Value> = arr
                .to_vec(cx)?
                .into_iter()
                .map(|v| Value::from_js(v, cx))
                .collect::<NeonResult<Vec<Value>>>()?;
            Ok(Value::Array(values))
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
