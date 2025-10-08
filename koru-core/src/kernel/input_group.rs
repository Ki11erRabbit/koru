use std::collections::HashMap;
use mlua::{IntoLua, UserData, UserDataFields, UserDataMethods};
use crate::key::KeyPress;

pub struct InputGroup {
    name: String,
    keys_to_command: HashMap<Vec<KeyPress>, String>,
}

impl InputGroup {
    pub fn new(name: impl Into<String>) -> Self {
        InputGroup {
            name: name.into(),
            keys_to_command: HashMap::new(),
        }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn add_key(&mut self, keys: Vec<KeyPress>, value: impl Into<String>) {
        self.keys_to_command.insert(keys, value.into());
    }
    
    pub fn remove_key(&mut self, keys: Vec<KeyPress>) {
        self.keys_to_command.remove(&keys);
    }
    
    pub fn get_command(&self, keys: &Vec<KeyPress>) -> Option<String> {
        self.keys_to_command.get(keys).cloned()
    }
}

impl UserData for InputGroup {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_function_get(
            "name",
            |_, this| {
                let this = this.borrow::<Self>()?;
                Ok(this.name.clone())
            }
        )
    }
    
    fn add_methods<F: UserDataMethods<Self>>(methods: &mut F) {
        methods.add_function(
            "get_command_or_key",
            |lua, this: &InputGroup, key_press: (Vec<KeyPress>,)| {
                match this.get_command(&key_press.0) {
                    Some(key) => key.into_lua(lua),
                    None => {
                        key_press.0.into_lua(lua)
                    }
                }
            }
        );
        methods.add_function_mut(
            "add_key",
            |lua, this: &mut InputGroup, (key_press, command): (Vec<KeyPress>, String)| {
                this.add_key(key_press, command);
                Ok(())
            }
        );
        methods.add_function_mut(
            "remove_key",
            |lua, this: &mut InputGroup, (key_press,): (Vec<KeyPress>,)| {
                this.remove_key(key_press);
                Ok(())
            }
        )
    }
}