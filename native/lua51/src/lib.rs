use common::{JsLuaState, register_module};

register_module!(mut m, {
    m.export_class::<JsLuaState>("LuaState")?;
    Ok(())
});
