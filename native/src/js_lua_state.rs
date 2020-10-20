use std::sync::Arc;
use std::{fs, thread};

use crate::js_traits::{FromJs, ToJs};
use crate::lua_execution;
use crate::value::Value;

use neon::context::Context;
use neon::handle::Handle;
use neon::prelude::*;
use rlua::Lua;

use neon::declare_types;

fn do_string_sync(
    mut cx: MethodContext<JsLuaState>,
    code: String,
    name: Option<String>,
) -> JsResult<JsValue> {
    let this = cx.this();
    let lua: &Lua = {
        let guard = cx.lock();
        let state = this.borrow(&guard);
        &state.lua.clone()
    };

    match lua_execution::do_string_sync(lua, code, name) {
        Ok(v) => v.to_js(&mut cx),
        Err(e) => cx.throw_error(e.to_string()),
    }
}

fn do_file_sync(
    mut cx: MethodContext<JsLuaState>,
    filename: String,
    chunk_name: Option<String>,
) -> JsResult<JsValue> {
    match fs::read_to_string(filename) {
        Ok(contents) => do_string_sync(cx, contents, chunk_name),
        Err(e) => cx.throw_error(e.to_string()),
    }
}

fn register_function<'a>(
    mut cx: MethodContext<'a, JsLuaState>,
    name: String,
    cb: Handle<JsFunction>,
) -> JsResult<'a, JsValue> {
    let this = cx.this();
    let handler = EventHandler::new(&cx, this, cb);
    let lua: &Lua = {
        let guard = cx.lock();
        let state = this.borrow(&guard);
        &state.lua.clone()
    };

    let callback = move |values: Vec<Value>| {
        let handler = handler.clone();

        thread::spawn(move || {
            handler.schedule_with(move |event_ctx, this, callback| {
                let arr = JsArray::new(event_ctx, values.len() as u32);
                // TODO remove unwraps, handle errors, and pass to callback if needed.
                for (i, value) in values.into_iter().enumerate() {
                    let js_val = value.to_js(event_ctx).unwrap();
                    arr.set(event_ctx, i as u32, js_val).unwrap();
                }
                // TODO is this how we handle passing the error?
                //  technically, this is an event emitter and not a callback, so it just shouldn't fire
                //  if there is an error. Not sure how to make it `on` event emitter vs multi-shot callback
                let args: Vec<Handle<JsValue>> = vec![event_ctx.null().upcast(), arr.upcast()];
                let _result = callback.call(event_ctx, this, args);
            });
        });
    };
    match lua_execution::register_function(lua, name, callback) {
        Ok(_) => Ok(cx.undefined().upcast()),
        Err(e) => cx.throw_error(e.to_string()),
    }
}

fn set_global<'a>(
    mut cx: MethodContext<'a, JsLuaState>,
    name: String,
    handle: Handle<'a, JsValue>,
) -> JsResult<'a, JsValue> {
    let this: Handle<JsLuaState> = cx.this();
    let lua: &Lua = {
        let guard = cx.lock();
        let state = this.borrow(&guard);
        &state.lua.clone()
    };
    let set_value = Value::from_js(handle, &mut cx)?;
    match lua_execution::set_global(lua, name, set_value) {
        Ok(v) => v.to_js(&mut cx),
        Err(e) => cx.throw_error(e.to_string()),
    }
}

fn get_global(mut cx: MethodContext<JsLuaState>, name: String) -> JsResult<JsValue> {
    let this: Handle<JsLuaState> = cx.this();
    let lua: &Lua = {
        let guard = cx.lock();
        let state = this.borrow(&guard);
        &state.lua.clone()
    };
    match lua_execution::get_global(lua, name) {
        Ok(v) => v.to_js(&mut cx),
        Err(e) => cx.throw_error(e.to_string()),
    }
}

pub struct LuaState {
    lua: Arc<Lua>,
}

impl LuaState {
    fn reset(&mut self) -> () {
        // By creating a new lua state, we remove all
        // references allowing the existing program to exit.
        self.lua = Arc::new(Lua::new());
    }
}

declare_types! {
    pub class JsLuaState for LuaState {
        init(_) {
            // TODO allow for newWith to allow for choosing included libraries
            Ok(LuaState {
                lua: Arc::new(Lua::new())
            })
        }

        method registerFunction(mut cx) {
            let name = cx.argument::<JsString>(0)?.value();
            let cb = cx.argument::<JsFunction>(1)?;
            register_function(cx, name, cb)
        }

        method reset(mut cx) {
            let mut this = cx.this();
            {
                let guard = cx.lock();
                let mut state = this.borrow_mut(&guard);
                state.reset();
            }
            Ok(cx.undefined().upcast())
        }

        method close(mut cx) {
            let mut this = cx.this();
            {
                let guard = cx.lock();
                let mut state = this.borrow_mut(&guard);
                state.reset();
            }
            Ok(cx.undefined().upcast())
        }

        method doStringSync(mut cx) {
            let code = cx.argument::<JsString>(0)?.value();
            let chunk_name = match cx.argument_opt(1) {
                Some(arg) => Some(arg.downcast::<JsString>().or_throw(&mut cx)?.value()),
                None => None
            };
            do_string_sync(cx, code, chunk_name)
        }

        method doFileSync(mut cx) {
            let filename = cx.argument::<JsString>(0)?.value();
            // TODO chop the filename on error a bit so it's legible.
            //  currently the `root/stuff/...` is at the end vs `.../stuff/things.lua`
            let chunk_name = match cx.argument_opt(1) {
                Some(arg) => Some(arg.downcast::<JsString>().or_throw(&mut cx)?.value()),
                None => Some(String::from(filename.clone()))
            };
            do_file_sync(cx, filename, chunk_name)
        }

        method setGlobal(mut cx) {
            let name = cx.argument::<JsString>(0)?.value();
            let value = cx.argument::<JsValue>(1)?;
            set_global(cx, name, value)
        }

        method getGlobal(mut cx) {
            let name = cx.argument::<JsString>(0)?.value();
            get_global(cx, name)
        }
    }
}
