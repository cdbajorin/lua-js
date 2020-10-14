use rlua::prelude::*;
use neon::prelude::*;

use crate::LuaState;
use rlua::{Error, MultiValue};
use std::collections::HashMap;

#[derive(Clone)]
pub enum Value {
    Undefined,
    String(String),
    Number(f64),
    Boolean(bool),
    Object(Vec<Value>, Vec<Value>),
    Array(Vec<Value>)
    // JsFunction(Handle<'a, JsFunction>)
}

impl ToLua<'_> for Value {
    fn to_lua(self, lua: rlua::Context<'_>) -> rlua::Result<LuaValue<'_>> {
        match self {
            Value::Undefined => Ok(LuaValue::Nil),
            Value::String(s) => {
                match lua.create_string(&s) {
                    Ok(s) => Ok(LuaValue::String(s)),
                    Err(e) => { panic!(e) }
                }
            }
            Value::Number(n) => Ok(LuaValue::Number(n)),
            Value::Boolean(b) => Ok(LuaValue::Boolean(b)),
            Value::Object(keys, values) => {
                todo!()
            }
        }
    }
}

impl FromLua<'_> for Value {
    fn from_lua(lua_value: LuaValue<'_>, lua: rlua::Context<'_>) -> rlua::Result<Self> {
        match lua_value {
            LuaValue::Nil => { Ok(Value::Undefined) }
            LuaValue::String(s) => {
                match s.to_str() {
                    Ok(s) => Ok(Value::String(s.to_owned())),
                    Err(e) => {
                        Err(e)
                    }
                }
            }
            LuaValue::Boolean(b) => Ok(Value::Boolean(b)),
            LuaValue::Integer(i) => Ok(Value::Number(i as f64)),
            LuaValue::Number(n) => Ok(Value::Number(n)),
            // Value::LightUserData(_) => {}
            // Value::Table(_) => {}
            // Value::Function(_) => {}
            // Value::Thread(_) => {}
            // Value::UserData(_) => {}
            // Value::Error(_) => {}
            _ => todo!()
        }
    }
}

impl Value {
    pub fn into_js<'a, T: Context<'a>>(self, cx: &mut T) -> Handle<'a, JsValue> {
        match self {
            // Intermediate::Undefined => cx.undefined().upcast(),
            Value::Undefined => cx.null().upcast(),
            Value::String(s) => cx.string(s).upcast(),
            Value::Number(n) => cx.number(n).upcast(),
            Value::Boolean(b) => cx.boolean(b).upcast(),
            Value::Object(keys, values) => {
                todo!()
            }
            _ => todo!()
        }
    }

    // TODO Error type
    pub fn from_js<'a, T: Context<'a>>(handle: Handle<'a, JsValue>, cx: &mut T) -> Result<Value, ()> {
        if handle.is_a::<JsUndefined>() || handle.is_a::<JsNull>() {
            Ok(Value::Undefined)
        } else if handle.is_a::<JsBoolean>() {
            let value = handle.downcast::<JsBoolean>().unwrap().value();
            Ok(Value::Boolean(value))
        } else if handle.is_a::<JsString>() {
            let value = handle.downcast::<JsString>().unwrap().value();
            Ok(Value::String(value))
        } else if handle.is_a::<JsNumber>() {
            let value = handle.downcast::<JsNumber>().unwrap().value();
            Ok(Value::Number(value))
        // } else if handle.is_a::<JsFunction>() {
        //     println!("A function?");
        //     Ok(LjsBridge::JsFunction(value))
        } else if handle.is_a::<JsObject>() {
            // TODO obviously, things inherit from object. Need to check for that type last.
            println!("an object?");
            todo!();
            // let obj = handle.downcast::<JsObject>()
            //     .unwrap_or(cx.empty_object());
            // let prop_names = obj.get_own_property_names()
            //     .unwrap_or(cx.empty_array());

            // let mut keys: Vec<Value> = vec![];
            // let mut values: Vec<Value> = vec![];
            // for i in 0..prop_names.len() {
            //     let key = prop_names.get(cx, i).unwrap();
            //     let value = obj.get(cx, key).unwrap();
            //     let k = Value::from_js(key, cx).unwrap();
            //     let v = Value::from_js(value, cx).unwrap();
            //     keys.push(k);
            //     values.push(v);
            // }
            // Ok(Value::Object(keys, values))
        } else if handle.is_a::<JsArray>() {
            todo!()
        } else {
            // TODO internal error type that covers js and lua errors
            Err(())
        }
    }
}