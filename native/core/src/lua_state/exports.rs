use std::cell::RefCell;
use std::fs;

use crate::lua_state::native::LuaState;
use crate::meta;
use crate::value::Value;

use mlua::StdLib;
use neon::prelude::*;

/// Container type that we pass back and forth to the JS runtime.
pub type JsLuaState = JsBox<RefCell<LuaState>>;

/// LuaState Class constructor
/// takes a single configuration object with the fields:
///  - `libraries` - an array of JS numbers corresponding to mlua::StdLib
pub fn luastate_constructor(mut cx: FunctionContext) -> JsResult<JsLuaState> {
    let opt_config = cx.argument_opt(0);
    if let None = opt_config {
        return Ok(cx.boxed(RefCell::new(LuaState::default())));
    };
    let options: Handle<JsObject> = opt_config
        .unwrap()
        .downcast_or_throw::<JsObject, _>(&mut cx)?;
    let libs: Handle<JsValue> = options.get(&mut cx, "libraries")?;
    let libraries = build_lua_stdlib(&mut cx, libs)?;
    let luastate = LuaState::new(libraries);
    Ok(cx.boxed(RefCell::new(luastate)))
}

/// LuaState Instance method `doStringSync(code: string, name?: string)`
/// compile and execute a string of lua code.
pub fn luastate_do_string_sync(mut cx: FunctionContext) -> JsResult<JsValue> {
    let luastate: Handle<JsLuaState> = cx.argument::<JsLuaState>(0)?;
    let code = cx.argument::<JsString>(1)?.value(&mut cx);
    let name = cx.argument::<JsString>(2)?.value(&mut cx);
    // TODO I believe using borrow_mut will give us run-time errors on race conditions?
    //  suggestion for doing try_borrow_mut.
    let luastate = luastate.borrow_mut();
    match luastate.do_string_sync(code, name) {
        Ok(v) => v.to_js(&mut cx),
        Err(e) => cx.throw_error(e.to_string()),
    }
}

/// LuaState instance method `doFileSync(filename: string, chunkName?: string)`
/// Optional chunkName is for error messaging.
pub fn luastate_do_file_sync(mut cx: FunctionContext) -> JsResult<JsValue> {
    let luastate: Handle<JsLuaState> = cx.argument::<JsLuaState>(0)?;
    let luastate = luastate.borrow_mut();
    let filepath: String = cx.argument::<JsString>(1)?.value(&mut cx);
    match fs::read_to_string(&filepath) {
        // TODO trim filepath for passing in as name
        Ok(code) => match luastate.do_string_sync(code, filepath) {
            Ok(v) => v.to_js(&mut cx),
            Err(e) => cx.throw_error(e.to_string()),
        },
        Err(e) => cx.throw_error(e.to_string()),
    }
}

/// LuaState instance method `reset()`
/// restarts the Lua runtime. In the future, this will also clear references held
/// by LuaState which might prevent the JS Runtime from closing (event emitters).
pub fn luastate_reset(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let luastate: Handle<JsLuaState> = cx.argument::<JsLuaState>(0)?;
    let mut luastate = luastate.borrow_mut();
    luastate.reset();
    Ok(cx.undefined())
}

/// LuaState instance method `setGlobal(name, value): boolean`
pub fn luastate_set_global(mut cx: FunctionContext) -> JsResult<JsValue> {
    let luastate: Handle<JsLuaState> = cx.argument::<JsLuaState>(0)?;
    let var_name: String = cx.argument::<JsString>(1)?.value(&mut cx);
    let value_handle = cx.argument(2)?;
    let value = Value::from_js(value_handle, &mut cx)?;
    let luastate = luastate.borrow_mut();

    match luastate.set_global(var_name, value) {
        Ok(v) => v.to_js(&mut cx),
        Err(e) => cx.throw_error(e.to_string()),
    }
}

/// LuaState instance method `getGlobal(name): T`
pub fn luastate_get_global(mut cx: FunctionContext) -> JsResult<JsValue> {
    let luastate: Handle<JsLuaState> = cx.argument::<JsLuaState>(0)?;
    let var_name: String = cx.argument::<JsString>(1)?.value(&mut cx);
    let luastate = luastate.borrow_mut();

    match luastate.get_global(var_name) {
        Ok(v) => v.to_js(&mut cx),
        Err(e) => cx.throw_error(e.to_string()),
    }
}

// Converts number values passed in from JS to mlua::StdLib values
fn flag_into_stdlib(flag: u32) -> Option<StdLib> {
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

// These correspond to our JS Enum. Used for a clearer error notification when including them in
// incompatible versions.
fn flag_to_string(flag: u32) -> String {
    const ALL_SAFE: u32 = u32::MAX - 1;
    match flag {
        0x1 => String::from("coroutine"),
        0x2 => String::from("table"),
        0x4 => String::from("io"),
        0x8 => String::from("os"),
        0x10 => String::from("string"),
        0x20 => String::from("utf8"),
        0x40 => String::from("bit"),
        0x80 => String::from("math"),
        0x100 => String::from("package"),
        0x200 => String::from("jit"),
        0x4000_0000 => String::from("ffi"),
        0x8000_0000 => String::from("debug"),
        ALL_SAFE => String::from("ALL_SAFE"),
        u32::MAX => String::from("ALL"),
        _ => flag.to_string(),
    }
}

// Converts `libraries` config property to a rust value for mlua.
fn build_lua_stdlib(cx: &mut FunctionContext, libs: Handle<JsValue>) -> NeonResult<StdLib> {
    if libs.is_a::<JsUndefined, _>(cx) {
        Ok(StdLib::ALL_SAFE)
    } else if libs.is_a::<JsArray, _>(cx) {
        let lib_flags = libs.downcast_or_throw::<JsArray, _>(cx)?.to_vec(cx)?;
        let mut lib_set = StdLib::NONE;
        for value in lib_flags.into_iter() {
            let flag = value.downcast_or_throw::<JsNumber, _>(cx)?.value(cx) as u32;

            if let Some(lib) = flag_into_stdlib(flag) {
                lib_set |= lib;
            } else {
                return cx.throw_error(format!(
                    "unrecognized Library flag \"{}\" for {}",
                    flag_to_string(flag),
                    meta::LUA_VERSION
                ));
            }
        }
        Ok(lib_set)
    } else {
        cx.throw_error("Expected 'libraries' to be an array")
    }
}
