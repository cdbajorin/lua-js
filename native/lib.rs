/// Shared export file for all of the lua versions.
/// Each version has their own Cargo.toml and directory, but
/// point to this file to build. This is done because there is
/// no CLI flag for a lib name. Without distinct lib names, the
/// binaries end up with the same symbols, regardless of the
/// feature flags being passed, and we end up with name clashes
/// when calling `require()` from node.
use common::*;
use neon::prelude::*;

#[neon::main]
fn main(mut m: ModuleContext) -> NeonResult<()> {
    m.export_function("LuaState_Constructor", luastate_constructor)?;
    m.export_function("LuaState_doStringSync", luastate_do_string_sync)?;
    m.export_function("LuaState_doFileSync", luastate_do_file_sync)?;
    m.export_function("LuaState_reset", luastate_reset)?;
    m.export_function("LuaState_getGlobal", luastate_get_global)?;
    m.export_function("LuaState_setGlobal", luastate_set_global)?;

    Ok(())
}
