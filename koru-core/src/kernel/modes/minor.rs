use mlua::Function;
use crate::kernel::input_group::InputGroup;
use crate::key::KeyPress;

pub struct MinorMode {
    input: InputGroup
}

impl MinorMode {
    pub fn new(input: InputGroup) -> Self {
        Self { input }
    }
    
    pub fn try_input(&self, keys: Vec<KeyPress>) -> Option<Function> {
        
    }
}