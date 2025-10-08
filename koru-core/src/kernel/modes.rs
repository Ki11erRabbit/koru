//! Modes are the Fundamental Units of Koru
//! 
//! Major modes define how the buffer will draw itself.
//! They also provide API that minor modes can consume.
//! 
//! Minor modes provide the input layer and issue commands to the major mode.
//! They can have their own commands and state.
mod major;
mod minor;

use mlua::{UserData, UserDataMethods};
pub use major::MajorMode;
pub use minor::MinorMode;
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

impl UserData for KeyBuffer {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function_mut(
            "clear",
            |lua, this: &mut KeyBuffer| {
                this.clear();
                Ok(())
            }
        );
        methods.add_function(
            "length",
            |_, this: &mut KeyBuffer| Ok(this.len().into())
        );
        methods.add_function_mut(
            "push",
            |_, this: &mut KeyBuffer, key: KeyPress| {
                this.push(key);
                Ok(())
            }
        );
    }
}

/// A command is a function that takes in a `KeyBuffer` to process the keypress
pub type Command = mlua::Function;