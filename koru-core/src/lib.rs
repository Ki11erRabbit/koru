mod modules;
mod kernel;

use std::sync::Arc;
use tokio::runtime::Builder;

pub use kernel::key;

pub trait Backend {
    fn shutdown(&self);
    fn get_keypress(&self) -> key::KeyPress;
    async fn get_keypress_async(&self) -> key::KeyPress;
}


pub fn koru_main(backend: Arc<dyn Backend>) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async {
        match kernel::start_kernel(backend,"koru.lua") {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    })?;
    
    Ok(())
}

