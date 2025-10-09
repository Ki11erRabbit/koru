mod utils;
mod state;
pub mod input;
mod lua_api;
mod files;
mod session;

use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use mlua::Lua;
use tokio::io::AsyncReadExt;
use tokio::task::JoinHandle;
use crate::Backend;


pub async fn start_kernel(backend: Arc<dyn Backend>) -> Result<(), Box<dyn Error>> {

    match utils::locate_config_path() {
        Some(config_path) => {
            state::set_config(config_path)
        }
        None => {
            return Err(Box::new(String::from("TODO: implement a first time wizard to set up the editor")));
        }
    }

    state::set_backend(backend.clone());

    backend.main_code().await?;

    backend.shutdown();
    
    Ok(())
}

pub async fn start_worker<P: AsRef<Path>>(worker_code_path: P) -> Result<(), Box<dyn Error>> {
    if !utils::does_file_exist(worker_code_path) {
        return Err(Box::new(String::from("Main thread code does not exist")));
    }

    let lua = Lua::new();

    lua.register_module(
        "Koru",
        lua_api::kernel_mod(&lua)?
    )?;

    let contents = {
        let mut contents = String::new();
        let mut file = tokio::fs::File::open(worker_code_path).await?;
        file.read_to_string(&mut contents).await?;
        contents
    };

    lua.load(contents.as_str()).exec_async().await?;
    Ok(())
}

pub async fn spawn_worker<P: AsRef<Path>>(worker_code_path: P) -> Result<JoinHandle<()>, Box<dyn Error>> {
    tokio::spawn(start_worker(worker_code_path))?
}