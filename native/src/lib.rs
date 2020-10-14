//! Main file handling exports, used for connecting JS to lua-js
// #![allow(unused)]

// #[macro_use]
// extern crate neon;

mod value;
// mod types;
mod error;
mod js;
// mod lua;

use crate::value::Value;

use std::thread;

use rlua::prelude::*;

use neon::borrow::Borrow;
use neon::context::Context;
use neon::event::EventHandler;
use neon::prelude::*;
// use neon::types::Value as JsValueTrait;

// this allows better type completion in CLion.
use neon::declare_types;
use neon::register_module;

use crate::js::{FromJs, ToJs};
use std::sync::Arc;
use std::fs;

// TODO single concern. Move the lua calls to the lua.rs file. re-implement the Intermediate Representation `Value`
fn do_string_sync<'a, CX: Context<'a>>(cx: &mut CX, lua: &Lua, code: String) -> JsResult<'a, JsValue> {
    lua.context(|lua_ctx| {
        let chunk = lua_ctx.load(&code);
        match chunk.exec() {
            Ok(_) => Ok(cx.undefined().upcast()),
            Err(e) => cx.throw_error(e.to_string())
        }
    })
}

fn do_file_sync<'a, CX: Context<'a>>(
    cx: &mut CX,
    lua: &Lua,
    filename: String,
    chunk_name: String,
) -> JsResult<'a, JsValue> {
    // let file_content = fs::read_to_string(filename).unwrap_or_else(|e| panic!(e.to_string()));
    let file_result = fs::read_to_string(filename);
    if let Err(e) = file_result {
        return cx.throw_error(e.to_string());
    };
    let file_content = file_result.unwrap();

    lua.context(|lua_ctx| {
        let chunk = lua_ctx.load(&file_content);
        match chunk.set_name(&chunk_name) {
            Ok(chunk) => {
                match chunk.exec() {
                    Ok(_) => Ok(cx.undefined().upcast()),
                    Err(e) => cx.throw_error(e.to_string())
                }
            }
            Err(e) => cx.throw_error(e.to_string())
        }
    })
}

/// Returns Unit. The value is passed to the JS callback.
fn register_function(cb: EventHandler, lua: &Lua, name: String) {
    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();
        let global_name = name.clone();

        let f = lua_ctx
            .create_function(move |c, args: LuaMultiValue| {
                let res_val: rlua::Result<Vec<Value>> =
                    args.into_iter().map(|v| Value::from_lua(v, c)).collect();
                let values = res_val.unwrap();
                let cb = cb.clone();

                thread::spawn(move || {
                    cb.schedule_with(move |ec, this, callback| {
                        let arr = JsArray::new(ec, values.len() as u32);
                        for (i, value) in values.into_iter().enumerate() {
                            let js_val = value.to_js(ec).unwrap();
                            arr.set(ec, i as u32, js_val).unwrap();
                        }
                        let args: Vec<Handle<JsValue>> = vec![arr.upcast()];
                        let _result = callback.call(ec, this, args);
                    });
                });
                Ok(())
            })
            .unwrap();
        // TODO unhandled result? What happens if it errors. Should this function return JsUndefined?
        let _r = globals.set(global_name, f);
        _r.unwrap()
    });
}

fn set_global<'a, CX: Context<'a>>(
    cx: &mut CX,
    lua: &Lua,
    name: String,
    handle: Handle<'a, JsValue>,
) -> JsResult<'a, JsValue> {
    match lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();
        let value = Value::from_js(handle, cx).unwrap();
        globals.set(name, value)
    }) {
        Ok(_) => Ok(cx.boolean(true).upcast()),
        Err(_) => Ok(cx.boolean(false).upcast()),
    }
}

fn get_global<'a, CX: Context<'a>>(cx: &mut CX, lua: &Lua, name: String) -> JsResult<'a, JsValue> {
    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();
        // TODO need to figure out error conversion to use ?;
        match globals.contains_key(name.clone()) {
            Ok(true) => {
                let lua_value: LuaValue = globals.get(name).unwrap();
                let value = Value::from_lua(lua_value, lua_ctx).unwrap();
                value.to_js(cx)
            },
            Ok(false) => Ok(cx.undefined().upcast()),
            Err(e) => {
                cx.throw_error(e.to_string())
            }
        }
    })
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
            register_function(cb, &lua, name);
            Ok(cx.undefined().upcast())
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
            let this = cx.this();
            let lua = {
                let guard = cx.lock();
                let state = this.borrow(&guard);
                state.lua.clone()
            };
            // Ok(do_string_sync(&mut cx, &lua, code).unwrap())
            match do_string_sync(&mut cx, &lua, code) {
                Ok(v) => Ok(v.upcast()),
                e => e
            }
        }

        method doFileSync(mut cx) {
            let filename = cx.argument::<JsString>(0)?.value();
            // TODO chop the filename on error a bit so it's legible.
            //  currently the `root/stuff/...` is at the end vs `.../stuff/things.lua`
            let chunk_name = match cx.argument_opt(1) {
                Some(arg) => arg.downcast::<JsString>().or_throw(&mut cx)?.value(),
                None => String::from(filename.clone())
            };
            let this = cx.this();
            let lua = {
                let guard = cx.lock();
                let state = this.borrow(&guard);
                state.lua.clone()
            };

            match do_file_sync(&mut cx, &lua, filename, chunk_name) {
                Ok(v) => Ok(v.upcast()),
                e => e
            }
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
            match set_global(&mut cx, &lua, name, value) {
                Ok(v) => Ok(v.upcast()),
                e => e
            }
        }

        method getGlobal(mut cx) {
            let name = cx.argument::<JsString>(0)?.value();
            let this = cx.this();
            let lua = {
                let guard = cx.lock();
                let state = this.borrow(&guard);
                state.lua.clone()
            };
            match get_global(&mut cx, &lua, name) {
                Ok(v) => Ok(v.upcast()),
                e => e
            }
        }
    }
}

register_module!(mut m, {
    m.export_class::<JsLuaState>("LuaState")?;
    Ok(())
});
