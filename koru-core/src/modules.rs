use mlua::{Lua, Table};
use mlua::prelude::{LuaResult, LuaTable};

mod data;

pub fn koru_mod(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;

    let package = exports.get::<Table>("package")?;
    let preload = package.get::<Table>("preload")?;
    preload.set(
        "KoruData",
        lua.create_function(|lua, ()| {
            data::koru_data(lua)
        })?
    )?;

    Ok(exports)
}