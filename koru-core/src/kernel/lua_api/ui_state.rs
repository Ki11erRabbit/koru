use mlua::{Lua, Table};

pub fn ui_module(lua: &Lua) -> mlua::Result<mlua::Table> {
    let exports = lua.create_table()?;
    
    exports.set(
        "set_ui_attr",
        lua.create_function(|lua, (name, value): (mlua::String, mlua::String)| {
            lua.globals().get::<Table>("__ui_attrs")?
                .set(name, value)?;
            Ok(())
        })?
    )?;
    
    Ok(exports)
}