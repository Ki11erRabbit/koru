mod modules;

use mlua::Lua;
use mlua::prelude::LuaResult;

pub fn koru_main() -> LuaResult<()> {
    let lua = Lua::new();
    
    lua.register_module(
        "Koru",
        koru_modules::koru_mod(&lua)?
    )?;
    
    lua.load("print('Hello, World!')").exec()?;
    Ok(())
}