mod modules;
mod kernel;

use std::sync::Arc;
use mlua::Lua;
use tokio::runtime::Builder;

pub trait Backend {
    fn shutdown(&self);
}


pub fn koru_main(backend: Arc<dyn Backend>) -> Result<(), Box<dyn std::error::Error>> {
    let lua = Lua::new();
    
    lua.register_module(
        "Koru",
        modules::koru_mod(&lua)?
    )?;
    
    lua.load("print('Hello, World!')").exec()?;

    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async {
        match kernel::start_kernel("koru") {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    })?;


    Ok(())
}

