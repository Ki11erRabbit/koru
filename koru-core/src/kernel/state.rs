use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use mlua::Lua;
use mlua::prelude::LuaTable;
use crate::{key, Backend};
use crate::kernel::input_group;

static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();
static BACKEND: OnceLock<Arc<dyn Backend>> = OnceLock::new();
static IMPLEMENTATION_LOCATION: OnceLock<PathBuf> = OnceLock::new();

pub fn set_config<P: AsRef<Path>>(path: P) {
    CONFIG_PATH.set(path.as_ref().to_path_buf()).expect("Config path already set");
}

pub fn set_backend(backend: Arc<dyn Backend>) {
    BACKEND.set(backend).expect("Backend already set");
}

pub fn get_backend() -> Arc<dyn Backend> {
    BACKEND.get().expect("Backend not set").clone()
}

pub fn set_impl_location<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    IMPLEMENTATION_LOCATION.set(path.to_owned()).expect("Implementation location already set");
}

pub fn get_impl_location() -> &'static PathBuf {
    IMPLEMENTATION_LOCATION.get().expect("Implementation location not set")
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
    exports.set(
        "get_keypress",
        lua.create_function(|_, _:()| {
            Ok(get_backend().get_keypress())
        })?
    )?;
    exports.set(
        "get_keypress_async",
        lua.create_async_function(|_, _: ()| {
            Ok(get_backend().get_keypress_async())
        })?
    )?;
    
    let package = exports.get::<mlua::Table>("package")?;
    let preload = package.get::<mlua::Table>("preload")?;
    preload.set(
        "Key",
        lua.create_function(|lua, ()| {
            key::key_module(lua)
        })?
    )?;
    preload.set(
       "InputGroup",
       lua.create_function(|lua, ()| {
           input_group::input_module(lua)
       })?
    )?;
    
    Ok(exports)
}