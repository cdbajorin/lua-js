//! Main file handling exports, used for connecting JS to lua-js
mod error;
mod js_lua_state;
mod js_traits;
mod lua_execution;
mod value;

pub use js_lua_state::JsLuaState;
pub use neon::register_module;

// register_module!(mut m, {
//     m.export_class::<JsLuaState>("LuaState")?;
//     Ok(())
// });
