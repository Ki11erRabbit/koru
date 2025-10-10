use mlua::Lua;
use mlua::prelude::LuaTable;
use crate::kernel::files::open_or_get_handle;

pub fn kernel_mod(lua: &Lua) -> mlua::Result<LuaTable> {
    let exports = lua.create_table()?;

    /*exports.set(
        "open_file",
        lua.create_async_function(async |lua, path: String| {
            let handle = open_or_get_handle(path).await.unwrap();
            lua.create_userdata(handle)
        })?,
    )?;*/

    let package = exports.get::<mlua::Table>("package")?;
    let preload = package.get::<mlua::Table>("preload")?;
    /*preload.set(
        "Key",
        lua.create_function(|lua, ()| {
            key::key_module(lua)
        })?
    )?;*/
    

    Ok(exports)
}