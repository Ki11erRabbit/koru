use mlua::UserData;
use crate::kernel::input_group::InputGroup;
use crate::kernel::modes::{Command, KeyBuffer};

pub struct MinorMode {
    input: InputGroup
}

impl MinorMode {
    pub fn new(input: InputGroup) -> Self {
        Self { input }
    }
    
    pub fn try_input(&self, keys: &KeyBuffer) -> Option<Command> {
        self.input.get_command(&keys)
    }
}

impl UserData for MinorMode {}

impl From