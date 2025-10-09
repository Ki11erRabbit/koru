use std::error::Error;
use std::sync::Arc;

use koru_core::{koru_main, Backend, InputSource};
use koru_core::kernel::input::{KeyBuffer, KeyPress};


struct TuirelmBackend {
    
}

impl TuirelmBackend {
    fn new() -> Self {
        TuirelmBackend {
            
        }
    }
}

impl Backend for TuirelmBackend {

    fn make_input_source(&self) -> Box<dyn InputSource> {
        Box::new(TuirelmInput { key_buffer: KeyBuffer::new() })
    }

    async fn main_code(&self) -> Result<(), Box<dyn Error>> {
        
        Ok(())
    }
}

struct TuirelmInput {
    key_buffer: KeyBuffer,
}

impl InputSource for TuirelmInput {
    async fn get_keypress_async(&mut self) {
        
    }

    async fn apply_key_buffer(&mut self, func: Box<dyn FnOnce(&KeyBuffer) -> bool>) {
        if func(&self.key_buffer) {
            self.key_buffer.clear();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let backend = TuirelmBackend::new();
    let backend = Arc::new(backend);
    koru_main(backend)
}