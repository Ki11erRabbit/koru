//! The Buffer is a window into the editor.
//!
//! The buffer provides an opaque space to allow for the editor to work with
//!

use std::error::Error;
use std::path::Path;
use std::thread::Builder;
use mlua::{Lua, Table, UserData, UserDataMethods, Value};
use crate::InputSource;
use crate::kernel::modes::{MajorMode, MinorMode};
use crate::key::KeyPress;

pub struct KeyBuffer {
    buffer: Vec<KeyPress>,
}

impl KeyBuffer {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn push(&mut self, key: KeyPress) {
        self.buffer.push(key);
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn get(&self) -> &[KeyPress] {
        &self.buffer
    }
}

pub struct Buffer {
    input_source: Box<dyn InputSource>,
    major_mode: MajorMode,
    minor_modes: Vec<MinorMode>,
    key_buffer: KeyBuffer,
}

impl Buffer {
    pub fn new(input_source: Box<dyn InputSource>, major_mode: MajorMode) -> Self {
        Buffer {
            input_source,
            major_mode,
            minor_modes: Vec::new(),
            key_buffer: KeyBuffer::new(),
        }
    }
    
    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let key = self.input_source.get_keypress_async().await;
            self.key_buffer.push(key);
            let commands = self.minor_modes.iter().map(|mm| {
                mm.try_input(&self.key_buffer)
            });
            let mut keys_consumed = false;
            for command in commands {
                let Some(command) = command else {
                    continue
                };
                keys_consumed = true;
                command.call(())?
            }
            if keys_consumed {
                self.key_buffer.clear();
            }
        }
    }
    
    pub async fn run_buffer(
        lua: &Lua,
        input_source: Box<dyn InputSource>, 
        major_mode: MajorMode,
        code: &str,
    ) -> Result<(), Box<dyn Error>> {
        let buffer = Buffer::new(input_source, major_mode);
        
        let exports = lua.create_table()?;
        let buffer_meta = lua.create_table()?;
        let buffer_constructor = lua.create_function(move |lua, () | {
            lua.create_userdata(buffer)
        })?;
        
        buffer_meta.set(
            "__call",
            buffer_constructor
        )?;
        exports.set_metatable(Some(buffer_meta))?; 
        
        lua.globals().set("Buffer", exports)?;
        
        
        lua.load(code).exec_async().await?;
        
        Ok(())
    }
}

impl UserData for Buffer {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function_mut(
            "run",
            |_, this: &mut Buffer, ()| {
                this.run()
            }
        );
    }
}
