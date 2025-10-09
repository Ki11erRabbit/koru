pub mod kernel;

use std::error::Error;
use std::sync::Arc;
use tokio::runtime::Builder;
use tokio::sync::Mutex;
use crate::kernel::input::KeyBuffer;

pub trait UiBackend: Send + Sync + 'static {
    fn main_code(&self) -> fn() -> Result<(), Box<dyn Error>>;
    
    fn input_events(&self) -> Result<Box<impl InputManager>, Box<dyn Error>>;
}
pub trait InputManager {
    async fn input_event(&mut self) -> Result<(), Box<dyn Error>>;
}


pub fn koru_main(backend: Arc<Mutex<impl UiBackend>>) -> Result<(), Box<dyn std::error::Error>> {
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

pub async fn koru_main_async(backend: Arc<Mutex<impl UiBackend>>) -> Result<(), Box<dyn std::error::Error>> {
    kernel::start_kernel(backend)?;

    Ok(())
}

