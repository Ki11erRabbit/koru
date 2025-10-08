use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use mlua::Lua;
use mlua::prelude::LuaTable;
use crate::Backend;

static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();
static BACKEND: OnceLock<Arc<dyn Backend>> = OnceLock::new();

pub fn set_config<P: AsRef<Path>>(path: P) {
    CONFIG_PATH.set(path.as_ref().to_path_buf()).expect("Config path already set");
}

pub fn set_backend(backend: Arc<dyn Backend>) {
    BACKEND.set(backend).expect("Backend already set");
}

pub fn get_backend() -> Arc<dyn Backend> {
    BACKEND.get().expect("Backend not set").clone()
}

pub fn kernel_mod(lua: &Lua) -> mlua::Result<LuaTable> {
    let exports = lua.create_table()?;
    
    exports.set(
        "get_config_path",
        lua.create_function(|_, _: ()| {
            let path = CONFIG_PATH.get().ok_or_else(mlua::Error::external)?;
            Ok(path.to_string_lossy())
        })?
    )?;
    exports.set(
        "shutdown",
        lua.create_function(|_, _:()| {
            get_backend().shutdown();
            Ok(())
        })?
    )?;
    
    Ok(exports)
}