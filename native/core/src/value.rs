//! Rust intermediate state between JS and Lua Value types.
use crate::js_traits::{FromJs, ToJs};
use mlua::prelude::{FromLua, Lua, LuaMultiValue, LuaValue, ToLua};

use neon::result::NeonResult;
use neon::types::{JsBoolean, JsNull, JsNumber, JsObject, JsString, JsUndefined, JsValue, JsFunction};
use neon::{context::Context, handle::Handle, object::Object};

const JS_MAX_SAFE_INTEGER: i64 = 9007199254740991;

#[derive(Debug, Clone)]
pub enum Value {
    // This lets us choose what our JS output is.
    // Both Null and Undefined convert to nil on the Lua side
    Null,
    Undefined,
    Boolean(bool),
    String(String),
    Integer(i64),
    Double(f64),
    // (k/v pairs, numerically indexed values)
    ObjectLike(Vec<(Value, Value)>, Vec<(Value, Value)>),
    Error(String),
}

impl Value {
    // TODO this is a hackaround for not implementing FromLuaMulti for Vec<Value>. Naming could be better?
    pub fn into_vec_for_lua_multi<'lua>(
        args: LuaMultiValue<'lua>,
        lua: &'lua Lua,
    ) -> mlua::Result<Vec<Value>> {
        args.into_vec()
            .into_iter()
            .map(|lua_v| Value::from_lua(lua_v, lua))
            .collect()
    }
}

impl<'lua> ToLua<'lua> for Value {
    fn to_lua(self, lua: &'lua Lua) -> mlua::Result<LuaValue<'lua>> {
        match self {
            Value::String(s) => {
                let lua_str = lua.create_string(s.as_bytes())?;
                Ok(LuaValue::String(lua_str))
            }
            Value::Integer(i) => Ok(LuaValue::Integer(i)),
            Value::Double(f) => Ok(LuaValue::Number(f)),
            Value::Undefined | Value::Null => Ok(LuaValue::Nil),
            Value::Boolean(b) => Ok(LuaValue::Boolean(b)),
            Value::Error(_err) => unimplemented!("ToLua for Error"),
            Value::ObjectLike(kv_pairs, array_like) => {
                let table = lua.create_table()?;
                for (i,v) in array_like.into_iter() {
                    // TODO consolidate flow through one of these.
                    if let Value::Double(idx) = i {
                        table.raw_set(idx+1.0, v)?;
                    } else if let Value::Integer(idx) = i {
                        table.raw_set(idx+1, v)?;
                    }
                }
                for (k, v) in kv_pairs.into_iter() {
                    table.raw_set(k, v)?;
                }
                Ok(LuaValue::Table(table))
            }
        }
    }
}

impl<'lua> FromLua<'lua> for Value {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> mlua::Result<Self> {
        match lua_value {
            LuaValue::Nil => Ok(Value::Null),
            LuaValue::Boolean(b) => Ok(Value::Boolean(b)),
            LuaValue::Integer(i) => {
                let int = i as i64;
                if int > JS_MAX_SAFE_INTEGER || int < -JS_MAX_SAFE_INTEGER {
                    Ok(Value::Integer(i as i64))
                } else {
                    Ok(Value::Double(i as f64))
                }
            },
            LuaValue::Number(f) => Ok(Value::Double(f)),
            LuaValue::String(s) => {
                let s = s.to_str()?;
                Ok(Value::String(s.to_owned()))
            }
            LuaValue::Table(table) => {
                // We're simulating an object/array similar to how Lua implements it.
                // we're using raw_len to avoid re-sizng the vec. I don't know if this is actually
                // an optimization, though.
                let len = table.raw_len();
                let mut kv_pairs: Vec<(Value, Value)> = Vec::with_capacity(len as usize);
                let mut indexed_values: Vec<(Value, Value)> = Vec::with_capacity(len as usize);

                for pair in table.pairs() {
                    let (key, value) = pair?;
                    let value_v = Value::from_lua(value, lua)?;
                    match key {
                        LuaValue::Integer(n) => {
                            let idx = n-1;
                            indexed_values.push((Value::Integer(idx), value_v))
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
                        // This handles all other cases where the property key is not representable
                        // in JS. TODO config option to ignore/drop/convert to symbol?
                        k @ _ => {
                            let err = mlua::Error::FromLuaConversionError {
                                from: &k.type_name(),
                                to: "JS Object PropertyKey",
                                message: None
                            };
                            return Err(err);
                        }
                    };
                };
                Ok(Value::ObjectLike(kv_pairs, indexed_values))
            }
            LuaValue::Function(_) => {
                Ok(Value::String("[LuaFunction]".to_string()))
            },
            LuaValue::Thread(_) => {
                Ok(Value::String("[LuaThread]".to_string()))
            },
            LuaValue::UserData(_) => unimplemented!("UserData"),
            LuaValue::LightUserData(_) => unimplemented!("LightUserData"),
            LuaValue::Error(e) => {
                // TODO what to do with error values instead of calls?
                Ok(Value::Error(e.to_string()))
            }
        }
    }
}

impl ToJs for Value {
    fn to_js<'a, CX: Context<'a>>(&self, cx: &mut CX) -> neon::result::JsResult<'a, JsValue> {
        match self {
            Value::String(s) => Ok(cx.string(s).upcast()),
            Value::Integer(int) => {
                let global = cx.global();
                let bigint_ctor = global.get(cx, "BigInt")?.downcast_or_throw::<JsFunction,_>(cx)?;
                let args = vec![cx.string(int.to_string())];
                let null = cx.null();
                let val = bigint_ctor.call(cx, null, args)?;
                Ok(val)
            },
            Value::Double(f) => Ok(cx.number(*f).upcast()),
            Value::Error(e) => cx.throw_error(e.to_string()),
            Value::Null => Ok(cx.null().upcast()),
            Value::Undefined => Ok(cx.undefined().upcast()),
            Value::Boolean(b) => Ok(cx.boolean(*b).upcast()),
            Value::ObjectLike(pairs, array_like) => {
                // let obj = cx.empty_object();
                let obj: Handle<JsObject> = if array_like.len() > 0 {
                    cx.empty_array().downcast_or_throw::<JsObject, CX>(cx)?
                } else {
                    cx.empty_object().downcast_or_throw::<JsObject, CX>(cx)?
                };

                for (idx, val) in array_like.into_iter() {
                    // TODO standardize treatment of keys.
                    let js_val = val.to_js(cx)?;
                    if let Value::Double(idx) = *idx {
                        obj.set(cx, idx as u32, js_val)?;
                    } else if let Value::Integer(idx) = *idx {
                        obj.set(cx, idx as u32, js_val)?;
                    }
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
            Ok(Value::Double(num))
        } else if handle.is_a::<JsString>() {
            let s = handle.downcast_or_throw::<JsString, CX>(cx)?.value();
            Ok(Value::String(s))
        } else if handle.is_a::<JsBoolean>() {
            let b = handle.downcast_or_throw::<JsBoolean, CX>(cx)?.value();
            Ok(Value::Boolean(b))
        } else if handle.is_a::<JsObject>() {

            if handle.is_a::<JsFunction>() {
                // TODO This should likely be an error. We want to direct the user towards
                //  using register_function/on("functionName") depending on sync/async.
                unimplemented!("JsFunction to Lua")
            } else {
                // JS Objects are lua-like, in that arrays can have object properties. If you set a
                // property using an integer, it will be treated as number like. x[1.5] will be coerced
                // to a string. So we can check for string/number/symbol and decide what do with it.
                let obj = handle.downcast_or_throw::<JsObject, CX>(cx)?;
                let props = obj.get_own_property_names(cx)?.to_vec(cx)?;

                // Don't know what's what, so we're just sizing each piece to the max we'd need
                let mut hash_like: Vec<(Value, Value)> = Vec::with_capacity(props.len() + 1);
                let mut array_like: Vec<(Value, Value)> = Vec::with_capacity(props.len() + 1);

                for key_handle in props {

                    if key_handle.is_a::<JsNumber>() {
                        // We can cast to usize because any property key set as a float will be cast
                        // to a string by the JS runtime.
                        let key = Value::from_js(key_handle, cx)?;
                        let value_handle = obj.get(cx, key_handle)?;
                        let value = Value::from_js(value_handle, cx)?;
                        array_like.push((key, value))
                    } else if key_handle.is_a::<JsString>() {
                        let key = Value::from_js(key_handle, cx)?;
                        let value_handle = obj.get(cx, key_handle)?;
                        let value = Value::from_js(value_handle, cx)?;
                        hash_like.push((key, value))
                    } else {
                        unimplemented!("JsSymbol Property Key")
                    }
                }
                Ok(Value::ObjectLike(hash_like, array_like))
            }
        } else {
            // TODO what should be happening here? returrning Err(Throw) is converted to nil instead of actually erroring out.
            panic!("Cannot convert JsValue from JS")
        }
    }
}
