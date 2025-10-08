use mlua::{Lua, Table};
use mlua::prelude::LuaResult;

mod utf8_buffer;


pub fn koru_data(lua: &Lua) -> LuaResult<Table> {
    let exports = lua.create_table()?;
    
    let package = exports.get::<Table>("package")?;
    let preload = package.get::<Table>("preload")?;
    preload.set(
        "Utf8Buffer",
        lua.create_function(|lua, ()| {
            utf8_buffer::utf8_buffer_mod(lua)
        })?
    )?;
    
    Ok(exports)
}