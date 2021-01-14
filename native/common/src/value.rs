/// Rust intermediate state between JS and Lua Value types.
use mlua::prelude::{FromLua, Lua, LuaMultiValue, LuaValue, ToLua};

use neon::result::NeonResult;
use neon::types::{
    JsArray, JsBoolean, JsFunction, JsNull, JsNumber, JsObject, JsString, JsUndefined, JsValue,
};
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
    // Numbers out of range of f64, either from JS bigint or from Lua Integer.
    Integer(i64),
    // JS Number values
    Double(f64),
    // Represents Tables/Objects/Arrays
    // (k/v pairs, numerically indexed values)
    ObjectLike(Vec<(Value, Value)>, Vec<(u32, Value)>),
    Error(String),
    // (description, whether or not use the registry)
    Symbol(String, bool),
}

impl Value {
    /// Convert LuaMulti into a value representing an array-like object.
    pub fn lua_multi_into_array<'lua>(
        args: LuaMultiValue<'lua>,
        lua: &'lua Lua,
    ) -> mlua::Result<Value> {
        let values: Vec<(u32, Value)> = args
            .into_vec()
            .into_iter()
            .enumerate()
            .map(|(idx, lua_v)| -> mlua::Result<(u32, Value)> {
                // TODO this isn't technically safe. Vararg size isn't limited to u32.
                Ok((idx as u32, Value::from_lua(lua_v, lua)?))
            })
            .collect::<mlua::Result<Vec<(u32, Value)>>>()?;
        Ok(Value::ObjectLike(vec![], values))
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
            Value::ObjectLike(kv_pairs, indexed_pairs) => {
                let table = lua.create_table()?;
                for (i, v) in indexed_pairs.into_iter() {
                    table.raw_set(i + 1, v)?;
                }
                for (k, v) in kv_pairs.into_iter() {
                    table.raw_set(k, v)?;
                }
                Ok(LuaValue::Table(table))
            }
            Value::Symbol(s, registry) => {
                if registry {
                    // TODO implement a LuaJsSymbol Registry? (config option?)
                    //  primitive table: `{ description: "symbol description" }` for use as keys.
                    unimplemented!("ToLua: Value::Symbol registry")
                } else {
                    let t = lua.create_table()?;
                    let key = lua.create_string("description")?;
                    let val = lua.create_string(&s)?;
                    t.set(key, val)?;
                    Ok(LuaValue::Table(t))
                }
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
            }
            LuaValue::Number(f) => Ok(Value::Double(f)),
            LuaValue::String(s) => {
                let s = s.to_str()?;
                Ok(Value::String(s.to_owned()))
            }
            LuaValue::Table(table) => {
                let len = table.raw_len();
                let mut kv_pairs: Vec<(Value, Value)> = Vec::with_capacity(len as usize);
                let mut indexed_values: Vec<(u32, Value)> = Vec::with_capacity(len as usize);

                for pair in table.pairs() {
                    let (key, value) = pair?;
                    let value = Value::from_lua(value, lua)?;
                    match key {
                        LuaValue::Integer(n) => {
                            // Lua Integers are "usually" i64, but JS array indices are limited to u32.
                            // So we're treating it numerically if it's within the (1,u32::MAX+1) range.
                            // otherwise, we're converting it to a string and treating it as an object key.
                            if n >= 1 && n <= (u32::MAX as i64) + 1 {
                                let idx = (n - 1) as u32;
                                indexed_values.push((idx, value))
                            } else if n == 0 {
                                // this is the special case for a few reasons. x["0"] === x[0] in JS, and lua[1] maps to js[0].
                                // a table like `{ [0] = 0, ["0"] = "0", "?" }` would have all 3 values mapped to the same zero-index in JS.
                                // So the numerical zero is being set as a symbol property with description ("LuaJs.0"), lua-1-index maps to zero-index,
                                // and string zero gets wrapped in single quotes down at at the match for LuaValue::String
                                kv_pairs.push((Value::Symbol("LuaJs.0".to_owned(), true), value))
                            } else {
                                let k = n.to_string();
                                kv_pairs.push((Value::String(k), value))
                            }
                        }
                        LuaValue::Number(f) => {
                            // floats are converted to strings, as that's the only representation
                            // that makes any sense on the JS side.
                            let float_key = f.to_string();
                            kv_pairs.push((Value::String(float_key), value));
                        }
                        LuaValue::String(s) => {
                            // If it's a i64, we're wrapping it in an extra single-quote.
                            // this handles { [-1] = -1, ["-1"] = "-1" } as well as { [1] = 1, ["1"] = "1" }
                            let mut key: String = s.to_str()?.to_owned();
                            if let Ok(num) = key.parse::<i64>() {
                                key = format!("'{}'", num);
                            }
                            kv_pairs.push((Value::String(key), value));
                        }
                        // This handles all other cases where the property key is not representable
                        // in JS. TODO config option to ignore/drop/convert to symbol?
                        k @ _ => {
                            let err = mlua::Error::FromLuaConversionError {
                                from: &k.type_name(),
                                to: "JS Object PropertyKey",
                                message: None,
                            };
                            return Err(err);
                        }
                    };
                }
                Ok(Value::ObjectLike(kv_pairs, indexed_values))
            }
            LuaValue::Function(_) => Ok(Value::String("[LuaFunction]".to_string())),
            LuaValue::Thread(_) => Ok(Value::String("[LuaThread]".to_string())),
            LuaValue::UserData(_) => unimplemented!("UserData"),
            LuaValue::LightUserData(_) => unimplemented!("LightUserData"),
            LuaValue::Error(e) => {
                // TODO is this even possible to get an error as a value from the runtime?
                //  `error()` converts to Result::Error.
                Ok(Value::Error(e.to_string()))
            }
        }
    }
}

/// JS From/Into
/// using `to_js` instead of `into_js` for consistency with lua traits.
impl Value {
    pub fn to_js<'a, CX: Context<'a>>(self, cx: &mut CX) -> neon::result::JsResult<'a, JsValue> {
        match self {
            Value::String(s) => Ok(cx.string(s).upcast()),
            Value::Integer(int) => {
                let global = cx.global();
                let bigint_ctor = global
                    .get(cx, "BigInt")?
                    .downcast_or_throw::<JsFunction, _>(cx)?;
                let args = vec![cx.string(int.to_string())];
                let null = cx.null();
                let val = bigint_ctor.call(cx, null, args)?;
                Ok(val)
            }
            Value::Double(f) => Ok(cx.number(f).upcast()),
            Value::Error(e) => cx.throw_error(e.to_string()),
            Value::Null => Ok(cx.null().upcast()),
            Value::Undefined => Ok(cx.undefined().upcast()),
            Value::Boolean(b) => Ok(cx.boolean(b).upcast()),
            Value::ObjectLike(kv_pairs, indexed_pairs) => {
                let obj: Handle<JsObject> = if indexed_pairs.len() > 0 {
                    cx.empty_array().downcast_or_throw::<JsObject, CX>(cx)?
                } else {
                    cx.empty_object().downcast_or_throw::<JsObject, CX>(cx)?
                };

                for (idx, val) in indexed_pairs.into_iter() {
                    let js_val = val.to_js(cx)?;
                    obj.set(cx, idx, js_val)?;
                }

                for (key, value) in kv_pairs {
                    let js_key = key.to_js(cx)?;
                    let js_value = value.to_js(cx)?;
                    obj.set(cx, js_key, js_value)?;
                }
                Ok(obj.upcast())
            }
            Value::Symbol(s, registry) => {
                let symbol_ctor = cx
                    .global()
                    .get(cx, "Symbol")?
                    .downcast_or_throw::<JsFunction, _>(cx)?;
                let null = cx.null();
                let args: Vec<Handle<JsValue>> = vec![cx.string(s).upcast()];
                if registry {
                    let symbol_for_registry = symbol_ctor
                        .get(cx, "for")?
                        .downcast_or_throw::<JsFunction, _>(cx)?;
                    symbol_for_registry.call(cx, null, args)
                } else {
                    symbol_ctor.call(cx, null, args)
                }
            }
        }
    }

    pub fn from_js<'a, CX: Context<'a>>(handle: Handle<JsValue>, cx: &mut CX) -> NeonResult<Self> {
        if handle.is_a::<JsNull, _>(cx) || handle.is_a::<JsUndefined, _>(cx) {
            Ok(Value::Null)
        } else if handle.is_a::<JsNumber, _>(cx) {
            let num = handle.downcast_or_throw::<JsNumber, _>(cx)?.value(cx);
            Ok(Value::Double(num))
        } else if handle.is_a::<JsString, _>(cx) {
            let s = handle.downcast_or_throw::<JsString, _>(cx)?.value(cx);
            Ok(Value::String(s))
        } else if handle.is_a::<JsBoolean, _>(cx) {
            let b = handle.downcast_or_throw::<JsBoolean, _>(cx)?.value(cx);
            Ok(Value::Boolean(b))
        } else if handle.is_a::<JsObject, _>(cx) {
            if handle.is_a::<JsFunction, _>(cx) {
                Ok(Value::String("[JsFunction]".to_owned()))
            } else if handle.is_a::<JsArray, _>(cx) {
                let arr: Handle<JsArray> = handle.downcast_or_throw::<JsArray, _>(cx)?;
                let arr_vec: Vec<Handle<JsValue>> = arr.to_vec(cx)?;
                let mut indexed_values: Vec<(u32, Value)> = vec![];
                for (idx, h) in arr_vec.into_iter().enumerate() {
                    let val = Value::from_js(h, cx)?;
                    // SAFETY: idx is coming from a conversion from JsArray, which
                    // means its indexes fit in u32.
                    indexed_values.push((idx as u32, val))
                }
                Ok(Value::ObjectLike(vec![], indexed_values))
            } else {
                let obj = handle.downcast_or_throw::<JsObject, _>(cx)?;
                let props = obj.get_own_property_names(cx)?.to_vec(cx)?;

                // Don't know what's what, so we're just sizing each piece to the max we could need
                let mut kv_pairs: Vec<(Value, Value)> = Vec::with_capacity(props.len() + 1);
                let mut indexed_values: Vec<(u32, Value)> = Vec::with_capacity(props.len() + 1);

                for key_handle in props.into_iter() {
                    if key_handle.is_a::<JsNumber, _>(cx) {
                        let key = key_handle.downcast_or_throw::<JsNumber, _>(cx)?.value(cx);
                        // SAFETY: Non-integer property keys get converted to strings when passed
                        // from the JS runtime. We're relying on that conversion to be correct.
                        let key = key as u32;
                        let value_handle = obj.get(cx, key_handle)?;
                        let value = Value::from_js(value_handle, cx)?;
                        indexed_values.push((key, value))
                    } else if key_handle.is_a::<JsString, _>(cx) {
                        let key = Value::from_js(key_handle, cx)?;
                        let value_handle = obj.get(cx, key_handle)?;
                        let value = Value::from_js(value_handle, cx)?;
                        kv_pairs.push((key, value))
                    } else {
                        unimplemented!("JsSymbol Property Key")
                    }
                }
                Ok(Value::ObjectLike(kv_pairs, indexed_values))
            }
        } else {
            cx.throw_error("Cannot convert JsValue from JS")
        }
    }
}
