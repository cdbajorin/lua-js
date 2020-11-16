use std::sync::Arc;
use std::{fs, thread};

use crate::js_traits::{FromJs, ToJs};
use crate::lua_execution;
use crate::value::Value;

use mlua::{Lua, StdLib};
use neon::context::Context;
use neon::handle::Handle;
use neon::prelude::*;

use neon::declare_types;

fn lua_version() -> &'static str {
    if cfg!(feature = "lua54") {
        "lua54"
    } else if cfg!(feature = "lua53") {
        "lua53"
    } else if cfg!(feature = "lua52") {
        "lua52"
    } else if cfg!(feature = "lua51") {
        "lua51"
    } else if cfg!(feature = "luajit") {
        "luajit"
    } else {
        panic!("No version specified")
    }
}

/// LuaState Class wrapper. Holds on to the lua context reference,
/// as well as the set of active lua libraries, and (eventually) the registered functions
pub struct LuaState {
    libraries: StdLib,
    lua: Arc<Lua>,
}

impl LuaState {
    fn reset(&mut self) -> () {
        // By creating a new lua state, we remove all references allowing the js runtime
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
            libraries: StdLib::ALL_SAFE,
            lua: Arc::new(Lua::new_with(StdLib::ALL_SAFE).unwrap()),
        }
    }
}

fn flag_into_std_lib(flag: u32) -> Option<StdLib> {
    const ALL_SAFE: u32 = u32::MAX - 1;
    match flag {
        #[cfg(any(feature = "lua54", feature = "lua53", feature = "lua52"))]
        0x1 => Some(StdLib::COROUTINE),
        0x2 => Some(StdLib::TABLE),
        0x4 => Some(StdLib::IO),
        0x8 => Some(StdLib::OS),
        0x10 => Some(StdLib::STRING),
        #[cfg(any(feature = "lua54", feature = "lua53"))]
        0x20 => Some(StdLib::UTF8),
        #[cfg(any(feature = "lua52", feature = "luajit"))]
        0x40 => Some(StdLib::BIT),
        0x80 => Some(StdLib::MATH),
        0x100 => Some(StdLib::PACKAGE),
        #[cfg(any(feature = "luajit"))]
        0x200 => Some(StdLib::JIT),
        #[cfg(any(feature = "luajit"))]
        0x4000_0000 => Some(StdLib::FFI),
        0x8000_0000 => Some(StdLib::DEBUG),
        u32::MAX => Some(StdLib::ALL),
        ALL_SAFE => Some(StdLib::ALL_SAFE),
        _ => None,
    }
}

/// These correspond to our JS Enum. Used for a clearer error notification when including them in
/// incompatible versions.
fn flag_to_string(flag: u32) -> String {
    const ALL_SAFE: u32 = u32::MAX - 1;
    match flag {
        0x1 => String::from("Coroutine"),
        0x2 => String::from("Table"),
        0x4 => String::from("Io"),
        0x8 => String::from("Os"),
        0x10 => String::from("String"),
        0x20 => String::from("Utf8"),
        0x40 => String::from("Bit"),
        0x80 => String::from("Math"),
        0x100 => String::from("Package"),
        0x200 => String::from("Jit"),
        0x4000_0000 => String::from("Ffi"),
        0x8000_0000 => String::from("Debug"),
        u32::MAX => String::from("All"),
        ALL_SAFE => String::from("AllSafe"),
        _ => flag.to_string(),
    }
}

fn build_libraries_option(
    mut cx: CallContext<JsUndefined>,
    libs: Handle<JsValue>,
) -> NeonResult<StdLib> {
    if libs.is_a::<JsArray>() {
        let libflags: Vec<Handle<JsValue>> = libs
            .downcast_or_throw::<JsArray, CallContext<JsUndefined>>(&mut cx)?
            .to_vec(&mut cx)?;

        // Hack to get a StdLib(0)
        let mut libset = StdLib::COROUTINE ^ StdLib::COROUTINE;
        for value in libflags.into_iter() {
            let flag = value
                .downcast_or_throw::<JsNumber, CallContext<JsUndefined>>(&mut cx)?
                .value() as u32;

            if let Some(lib) = flag_into_std_lib(flag) {
                libset |= lib;
            } else {
                return cx.throw_error(format!(
                    "unrecognized Library flag \"{}\" for {}",
                    flag_to_string(flag),
                    lua_version()
                ));
            }
        }
        Ok(libset)
    } else if libs.is_a::<JsUndefined>() {
        Ok(StdLib::ALL_SAFE)
    } else {
        cx.throw_error("Expected 'libraries' to be an an array")
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

fn call_chunk<'a>(
    mut cx: MethodContext<'a, JsLuaState>,
    code: String,
    chunk_name: Option<String>,
    js_args: Handle<'a, JsArray>,
) -> JsResult<'a, JsValue> {
    let this = cx.this();
    let mut args: Vec<Value> = vec![];
    let js_args = js_args.to_vec(&mut cx)?;
    for arg in js_args.iter() {
        let value = Value::from_js(*arg, &mut cx)?;
        args.push(value);
    }
    let lua: &Lua = {
        let guard = cx.lock();
        let state = this.borrow(&guard);
        &state.lua.clone()
    };
    match lua_execution::call_chunk(&lua, code, chunk_name, args) {
        Ok(v) => v.to_js(&mut cx),
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

        method callChunk(mut cx) {
            let code = cx.argument::<JsString>(0)?.value();
            let (chunk_name, args) = match cx.len() {
                2 => {
                    let args = cx.argument::<JsArray>(1)?;
                    Ok((None, args))
                },
                3 => {
                    let chunk_name = cx.argument::<JsString>(1)?.value();
                    let args = cx.argument::<JsArray>(2)?;
                    Ok((Some(chunk_name), args))
                },
                _ => {
                    let e = cx.string(format!("expected 2 or 3 arguments. Found: {}", cx.len()));
                    cx.throw(e)
                }
            }?;
            call_chunk(cx, code, chunk_name, args)
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
