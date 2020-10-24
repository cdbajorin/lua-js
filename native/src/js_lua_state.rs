use std::sync::Arc;
use std::{fs, thread};

use crate::js_traits::{FromJs, ToJs};
use crate::lua_execution;
use crate::value::Value;

use neon::context::Context;
use neon::handle::Handle;
use neon::prelude::*;
use rlua::{Lua, StdLib};

use neon::declare_types;

/// LuaState Class wrapper. Holds on to the lua context reference,
/// as well as the set of active lua libraries, and (eventually) the registered functions
pub struct LuaState {
    libraries: StdLib,
    lua: Arc<Lua>,
}

impl LuaState {
    fn reset(&mut self) -> () {
        // By creating a new lua state, we remove all references allowing the program
        // to exit if we've attached any event emitters. Without this, the program won't
        // close. Is there a more explicit way to close event listeners, or is relying on
        // the GC a normal/reasonable approach?
        let lua = unsafe { Lua::unsafe_new_with(self.libraries) };
        self.lua = Arc::new(lua)
    }
}

impl Default for LuaState {
    fn default() -> Self {
        LuaState {
            libraries: StdLib::ALL_NO_DEBUG,
            lua: Arc::new(Lua::new_with(StdLib::ALL_NO_DEBUG)),
        }
    }
}

fn build_libraries_option(
    mut cx: CallContext<JsUndefined>,
    libs: Handle<JsValue>,
) -> NeonResult<StdLib> {
    // flag_set is for throwing errors to notify the user of a bad bitflag set.
    let (flags, flag_set) = if libs.is_a::<JsArray>() {
        let libflags = libs
            .downcast_or_throw::<JsArray, CallContext<JsUndefined>>(&mut cx)?
            .to_vec(&mut cx)?;

        let mut flag_count: u32 = 0;
        let mut flag_set: Vec<String> = vec![];
        for value in libflags.into_iter() {
            let flag = value
                .downcast_or_throw::<JsNumber, CallContext<JsUndefined>>(&mut cx)?
                .value() as u32;
            flag_count += flag;
            let flag_str = format!("{}", flag);
            flag_set.push(flag_str);
        }
        (flag_count, flag_set)
    } else {
        (0 as u32, vec![])
    };
    match StdLib::from_bits(flags) {
        None => {
            let flag_set_str = flag_set.join(", ");
            let throw_msg = cx.string(format!(
                "Cannot find libraries associated with bitflag set [{}]",
                flag_set_str
            ));
            cx.throw(throw_msg)
        }
        Some(v) => Ok(v),
    }
}

fn init(mut cx: CallContext<JsUndefined>) -> NeonResult<LuaState> {
    let opt_options = cx.argument_opt(0);

    if let None = opt_options {
        return Ok(LuaState::default());
    };
    let options: Handle<JsObject> = opt_options.unwrap().downcast_or_throw(&mut cx)?;
    let libraries_key = cx.string("libraries");
    let libs = options.get(&mut cx, libraries_key)?;
    let libraries = build_libraries_option(cx, libs)?;

    // Because we're allowing the end user to dynamically choose their libraries,
    // we're using the unsafe call in case they include `debug`. We need to notify
    // the end user in the documentation about the caveats of `debug`.
    let lua = unsafe {
        let lua = Lua::unsafe_new_with(libraries);
        Arc::new(lua)
    };
    Ok(LuaState { lua, libraries })
}

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
                // TODO How to pass an error via on('error') vs the current setup?
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

declare_types! {
    pub class JsLuaState for LuaState {

        init(cx) {
            init(cx)
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
