//! Main file handling exports, used for connecting JS to lua-js
mod error;
mod js_traits;
mod lua_execution;
mod js_lua_state;
mod value;

use js_lua_state::JsLuaState;

use neon::register_module;

register_module!(mut m, {
    m.export_class::<JsLuaState>("LuaState")?;
    Ok(())
});
