use std::collections::HashMap;
use mlua::{Function, IntoLua, Lua, UserData, UserDataFields, UserDataMethods};
use crate::kernel::modes::Command;
use crate::key::KeyPress;

pub struct InputGroup {
    name: String,
    keys_to_command: HashMap<Vec<KeyPress>, Command>,
    default_command: Option<Command>,
}

impl InputGroup {
    pub fn new(name: impl Into<String>) -> Self {
        InputGroup {
            name: name.into(),
            keys_to_command: HashMap::new(),
            default_command: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_key(&mut self, keys: Vec<KeyPress>, function: Command) {
        self.keys_to_command.insert(keys, function);
    }

    pub fn remove_key(&mut self, keys: Vec<KeyPress>) {
        self.keys_to_command.remove(&keys);
    }

    pub fn get_command(&self, keys: &Vec<KeyPress>) -> Option<Command> {
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
        );
        fields.add_field_method_set(
            "default_command",
            |_, this, function: Function| {
                this.default_command = Some(function);
                Ok(())
            }
        );
    }

    fn add_methods<F: UserDataMethods<Self>>(methods: &mut F) {
        methods.add_function(
            "get_command",
            |lua, this: &InputGroup, key_press: (Vec<KeyPress>,)| {
                match this.get_command(&key_press.0) {
                    Some(key) => key.into_lua(lua),
                    None => {
                        this.default_command.as_ref().cloned().into_lua(lua)
                    }
                }
            }
        );
        methods.add_function_mut(
            "add_command",
            |lua, this: &mut InputGroup, (key_press, command): (Vec<KeyPress>, Command)| {
                this.add_key(key_press, command);
                Ok(())
            }
        );
        methods.add_function_mut(
            "remove_command",
            |lua, this: &mut InputGroup, (key_press,): (Vec<KeyPress>,)| {
                this.remove_key(key_press);
                Ok(())
            }
        );
    }
}

pub fn input_module(lua: &Lua) -> mlua::Result<mlua::Table> {
    let exports = lua.create_table()?;

    let meta = lua.create_table()?;

    meta.set(
        "__call",
        lua.create_function(|lua, string: String| {
            let input = InputGroup::new(string);
            lua.create_userdata(input)
        })?
    )?;
    exports.set_metatable(Some(meta))?;

    Ok(exports)
}