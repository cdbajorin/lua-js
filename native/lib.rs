/// Shared export file for each of the lua versions.
/// Each version has their own Cargo.toml and directory, but
/// point to this file to build. This is done because there is
/// no CLI flag for a lib name. Without distinct lib names, the
/// binaries end up with the same symbols, regardless of the
/// feature flags being passed, and we end up with name clashes
/// when calling `require()` from node.
use core::{JsLuaState, register_module};

register_module!(mut m, {
    m.export_class::<JsLuaState>("LuaState")
});
