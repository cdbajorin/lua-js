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

fn do_string_sync<'a, CX: Context<'a>>(
    cx: &mut CX,
    lua: &Lua,
    code: String,
    name: Option<String>,
) -> JsResult<'a, JsValue> {
    match lua_execution::do_string_sync(lua, code, name) {
        Ok(v) => v.to_js(cx),
        Err(e) => cx.throw_error(e.to_string()),
    }
}

fn do_file_sync<'a, CX: Context<'a>>(
    cx: &mut CX,
    lua: &Lua,
    filename: String,
    chunk_name: Option<String>,
) -> JsResult<'a, JsValue> {
    match fs::read_to_string(filename) {
        Ok(contents) => do_string_sync(cx, lua, contents, chunk_name),
        Err(e) => cx.throw_error(e.to_string()),
    }
}

fn register_function<'a, CX: Context<'a>>(
    cx: &mut CX,
    handler: EventHandler,
    lua: &Lua,
    name: String,
) -> JsResult<'a, JsValue> {
    let callback = move |values: Vec<Value>| {
        let handler = handler.clone();
        thread::spawn(move || {
            handler.schedule_with(move |event_ctx, this, callback| {
                let arr = JsArray::new(event_ctx, values.len() as u32);
                for (i, value) in values.into_iter().enumerate() {
                    let js_val = value.to_js(event_ctx).unwrap();
                    arr.set(event_ctx, i as u32, js_val).unwrap();
                }
                let args: Vec<Handle<JsValue>> = vec![arr.upcast()];
                let _result = callback.call(event_ctx, this, args);
            });
        });
    };
    match lua_execution::register_function(lua, name, callback) {
        Ok(_) => Ok(cx.undefined().upcast()),
        Err(e) => cx.throw_error(e.to_string()),
    }
}

fn set_global<'a, CX: Context<'a>>(
    cx: &mut CX,
    lua: &Lua,
    name: String,
    handle: Handle<'a, JsValue>,
) -> JsResult<'a, JsValue> {
    let set_value = Value::from_js(handle, cx)?;
    match lua_execution::set_global(lua, name, set_value) {
        Ok(v) => v.to_js(cx),
        Err(e) => cx.throw_error(e.to_string()),
    }
}

fn get_global<'a, CX: Context<'a>>(cx: &mut CX, lua: &Lua, name: String) -> JsResult<'a, JsValue> {
    match lua_execution::get_global(&lua, name) {
        Ok(v) => v.to_js(cx),
        Err(e) => cx.throw_error(e.to_string()),
    }
}

pub struct LuaState {
    cb: Option<EventHandler>,
    lua: Arc<Lua>,
}

impl LuaState {
    fn reset(&mut self) -> () {
        self.cb = None;
        // By creating a new lua state, we remove all
        // references allowing the existing program to exit.
        self.lua = Arc::new(Lua::new());
    }
}

declare_types! {
    pub class JsLuaState for LuaState {
        init(_) {
            // TODO allow for newWith to allow for choosing libraries
            Ok(LuaState {
                cb: None,
                lua: Arc::new(Lua::new())
            })
        }

        method registerFunction(mut cx) {
            let name = cx.argument::<JsString>(0)?.value();
            let f = cx.argument::<JsFunction>(1)?;
            let this = cx.this();
            let cb = EventHandler::new(&cx, this, f);
            let lua = {
                let guard = cx.lock();
                let state = this.borrow(&guard);
                state.lua.clone()
            };

            register_function(&mut cx, cb, &lua, name)
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
            let this = cx.this();
            let lua = {
                let guard = cx.lock();
                let state = this.borrow(&guard);
                state.lua.clone()
            };
            do_string_sync(&mut cx, &lua, code, chunk_name)
        }

        method doFileSync(mut cx) {
            let filename = cx.argument::<JsString>(0)?.value();
            // TODO chop the filename on error a bit so it's legible.
            //  currently the `root/stuff/...` is at the end vs `.../stuff/things.lua`
            let chunk_name = match cx.argument_opt(1) {
                Some(arg) => Some(arg.downcast::<JsString>().or_throw(&mut cx)?.value()),
                None => Some(String::from(filename.clone()))
            };
            let this = cx.this();
            let lua = {
                let guard = cx.lock();
                let state = this.borrow(&guard);
                state.lua.clone()
            };
            do_file_sync(&mut cx, &lua, filename, chunk_name)
        }

        method setGlobal(mut cx) {
            let name = cx.argument::<JsString>(0)?.value();
            let value = cx.argument::<JsValue>(1)?;
            let this = cx.this();
            let lua = {
                let guard = cx.lock();
                let state = this.borrow(&guard);
                state.lua.clone()
            };
            set_global(&mut cx, &lua, name, value)
        }

        method getGlobal(mut cx) {
            let name = cx.argument::<JsString>(0)?.value();
            let this = cx.this();
            let lua = {
                let guard = cx.lock();
                let state = this.borrow(&guard);
                state.lua.clone()
            };
            get_global(&mut cx, &lua, name)
        }
    }
}
