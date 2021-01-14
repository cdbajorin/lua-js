use crate::error::Result;
use crate::value::Value;

use mlua::{FromLua, Lua, StdLib};
use neon::prelude::Finalize;

pub struct LuaState {
    lua: Lua,
    libraries: StdLib,
}

fn stdlib_contains_unsafe(libs: StdLib) -> bool {
    #[cfg(feature = "luajit")]
    if libs.contains(StdLib::FFI) || libs.contains(StdLib::JIT) {
        return true;
    };
    libs.contains(StdLib::DEBUG)
}

impl LuaState {
    fn init_lua_with_libraries(libraries: StdLib) -> Lua {
        // Lua internals has some error protection around passing binary into lua load()
        // it keeps a `safe` flag internally, so we're utilizing its own safety checks by
        // branching on `new_with`.
        if stdlib_contains_unsafe(libraries) {
            unsafe { Lua::unsafe_new_with(libraries) }
        } else {
            // This can be safely unwrapped because errors stem from either initiating with
            // unsafe libraries (which is handled above), or mlua itself is broken which should
            // panic anyway.
            Lua::new_with(libraries).unwrap()
        }
    }

    pub fn new(libraries: StdLib) -> Self {
        let lua = LuaState::init_lua_with_libraries(libraries);
        LuaState { lua, libraries }
    }

    pub fn do_string_sync(&self, code: String, name: String) -> Result<Value> {
        let chunk = self.lua.load(&code);
        let chunk = chunk.set_name(&name)?;
        match chunk.call(()) {
            Ok(values) => Ok(Value::lua_multi_into_array(values, &self.lua)?),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_global(&self, key: String, value: Value) -> Result<Value> {
        let globals = self.lua.globals();
        globals.set(key, value)?;
        Ok(Value::Boolean(true))
    }

    pub fn get_global(&self, key: String) -> Result<Value> {
        let globals = self.lua.globals();
        if let Ok(true) = globals.contains_key(key.as_str()) {
            let lua_value = globals.get(key)?;
            return Ok(Value::from_lua(lua_value, &self.lua)?);
        };
        Ok(Value::Undefined)
    }

    pub fn reset(&mut self) {
        self.lua = LuaState::init_lua_with_libraries(self.libraries);
    }
}

impl Default for LuaState {
    fn default() -> Self {
        let libraries = StdLib::ALL_SAFE;
        LuaState::new(libraries)
    }
}

impl Finalize for LuaState {
    // TODO we don't need to do anything until we start storing persistent functions
    // fn finalize<'a, C: Context<'a>>(self, _cx: &mut C) {
    // }
}
