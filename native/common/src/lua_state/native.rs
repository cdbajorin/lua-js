// use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;
use crate::value::Value;

use mlua::{FromLua, Lua, StdLib};
use neon::prelude::*;

pub struct LuaState {
    lua: Lua,
    libraries: StdLib,
    // stored_functions: HashMap<String, Arc<Root<JsFunction>>>,
    event_listeners: Vec<Arc<Root<JsFunction>>>,
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
        // let stored_functions = HashMap::new();
        let event_listeners = vec![];

        LuaState {
            lua,
            libraries,
            // stored_functions,
            event_listeners
        }
    }

    pub fn do_string_sync(&self, code: String, name: String) -> Result<Value> {
        let chunk = self.lua.load(&code);
        let chunk = chunk.set_name(&name)?;
        match chunk.call(()) {
            Ok(values) => Ok(Value::lua_multi_into_js_array(values, &self.lua)?),
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
        // self.stored_functions = HashMap::new();
    }

    pub fn register_event_listener(&mut self, queue: EventQueue, varname: String, f: Root<JsFunction>) -> Result<Value> {
        let callback = Arc::new(f);
        // Store the Rooted function so we can cleanup in Finalize. Because Lua is 'static, and we're moving
        // `queue` into Lua function, our only guarantee as to when this is no longer used is when the consumer
        // explicitly calls close/reset which drops all the `lua_f` functions, which will allow finalize/GC to run.
        self.event_listeners.push(Arc::clone(&callback));

        let lua_f = self.lua.create_function(move |lua, args| {
            // Clone the callback into the lua function scope. This allows us to try_unwrap without
            // directly pointing to the outside one, which gets dropped at the end of register_event_listener.
            // we can then refer to this one within `queue.send`.
            let callback = Arc::clone(&callback);
            let values = Value::lua_multi_into_vec(args, lua)?;
            queue.send(move |mut cx| {
                let this = cx.undefined();
                let cb = Arc::try_unwrap(callback)
                    .or_else(|cb| Ok((*cb).clone(&mut cx)))?
                    .into_inner(&mut cx);

                let args: Vec<Handle<JsValue>> = values
                    .into_iter()
                    .map(|v| Ok(v.to_js(&mut cx)?))
                    .collect::<NeonResult<Vec<Handle<JsValue>>>>()?;
                let _ = cb.call(&mut cx, this, args)?;
                Ok(())
            });
            Ok(Value::Null)
        })?;

        let globals = self.lua.globals();
        globals.set(varname, lua_f)?;
        Ok(Value::Undefined)
    }
}

impl Finalize for LuaState {
    fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
        // Cleanup handles to event listener callbacks
        for cb in self.event_listeners {
            let root = Arc::try_unwrap(cb)
                .expect("root still borrowed");
            root.into_inner(cx);
        }

        // Cleanup handles to Persistent callbacks.
        // for (_, f) in self.stored_functions {
        //     dbg!("Finalizing");
        //     f.finalize(cx);
        // }
    }
}
