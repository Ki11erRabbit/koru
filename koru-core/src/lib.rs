pub mod kernel;

use std::error::Error;
use std::sync::Arc;
use tokio::runtime::Builder;


use crate::kernel::input::KeyBuffer;

pub trait Backend {
    fn shutdown(&self);
    fn make_input_source(&self) -> Box<dyn InputSource>;
    async fn main_code(&self) -> Result<(), Box<dyn Error>>;
}

pub trait InputSource: Send {
    async fn get_keypress_async(&self) -> KeyBuffer;
}


pub fn koru_main(backend: Arc<dyn Backend>) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async {
        match kernel::start_kernel(backend) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    })?;

    Ok(())
}

