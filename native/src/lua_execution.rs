//! Connection point from lua-js to rlua itself.
use crate::error::Result;
use crate::value::Value;
use rlua::prelude::LuaValue;
use rlua::{FromLua, Function, Lua, MultiValue, ToLua};

pub fn do_string_sync(lua: &Lua, code: String, chunk_name: Option<String>) -> Result<Value> {
    lua.context(|ctx| {
        let chunk = ctx.load(&code);
        let named_chunk = match chunk_name {
            None => Ok(chunk),
            Some(name) => chunk.set_name(&name),
        }?;
        match named_chunk.exec() {
            Ok(_) => Ok(Value::Undefined),
            Err(e) => Err(e.into()),
        }
    })
}

pub fn call_chunk(
    lua: &Lua,
    code: String,
    chunk_name: Option<String>,
    args: Vec<Value>,
) -> Result<Value> {
    let r = lua.context(|ctx| {
        let chunk = ctx.load(&code);
        let named_chunk = match chunk_name {
            None => Ok(chunk),
            Some(name) => chunk.set_name(&name),
        }?;
        let f: Function = named_chunk.eval()?;
        let lua_args: Vec<LuaValue> = args
            .into_iter()
            .map(|value| value.to_lua(ctx))
            .collect::<rlua::Result<Vec<LuaValue>>>()?;
        let r = f.call(MultiValue::from_vec(lua_args))?;
        Value::from_lua(r, ctx)
    })?;
    Ok(r)
}

pub fn get_global(lua: &Lua, name: String) -> Result<Value> {
    lua.context(|ctx| {
        let globals = ctx.globals();
        let has_key = globals.contains_key(name.clone())?;
        match has_key {
            true => {
                let lua_value = globals.get(name)?;
                let value = Value::from_lua(lua_value, ctx)?;
                Ok(value)
            }
            false => Ok(Value::Undefined),
        }
    })
}

pub fn set_global(lua: &Lua, name: String, value: Value) -> Result<Value> {
    lua.context(|ctx| {
        let globals = ctx.globals();
        let _ = globals.set(name, value)?;
        Ok(Value::Boolean(true))
    })
}

// TODO not sure how else to approach this regarding EventHandler
pub fn register_function<F: 'static + Send + Sync + Fn(Vec<Value>) -> ()>(
    lua: &Lua,
    name: String,
    callback: F,
) -> Result<Value> {
    lua.context(|ctx| {
        let globals = ctx.globals();
        // TODO if this function fails, it should be passed to an event emitter, on("error")
        let f = ctx.create_function(move |c, args: MultiValue| {
            let values = Value::into_vec_for_lua_multi(args, c)?;
            // TODO this should have an error CB?
            callback(values);
            Ok(Value::Undefined)
        })?;
        let _ = globals.set(name, f)?;
        Ok(Value::Undefined)
    })
}
