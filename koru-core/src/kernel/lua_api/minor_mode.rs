use std::collections::HashMap;
use mlua::prelude::LuaUserData;
use mlua::{AnyUserData, Table, UserDataMethods};
use crate::kernel::input::KeyPress;
use crate::kernel::lua_api::Command;
use crate::keybinding::Keybinding;

pub struct MinorMode {
    commands: Vec<Command>,
    aliases: HashMap<String, usize>,
    keybinding: Keybinding<String>
}

impl MinorMode {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            aliases: HashMap::new(),
            keybinding: Keybinding::new()
        }
    }

    pub fn register_command(&mut self, name: String, command: Command) {
        let index = self.commands.len();
        self.commands.push(command);
        self.aliases.insert(name, index);
    }

    pub fn register_alias(&mut self, name: impl AsRef<str>, alias: String) {
        if let Some(index) = self.aliases.get(name.as_ref()) {
            self.aliases.insert(alias, *index);
        }
    }
    
    pub fn add_keybinding(&mut self, key_seq: Vec<KeyPress>, binding: String) {
        self.keybinding.add_binding(key_seq, binding);
    }
    
    pub fn get_keybinding(&self, key_seq: &[KeyPress]) -> Option<String> {
        self.keybinding.lookup(key_seq)
    }
}

impl LuaUserData for MinorMode {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "register_command",
            |_, this, (arg,): (AnyUserData,)| {
                let command = arg.take::<Command>()?;
                this.register_command(command.name.clone(), command);
                Ok(())
            }
        );
        methods.add_method_mut(
            "register_alias",
            |_, this, (command_name, alias): (mlua::String, mlua::String)| {
                let command_name = command_name.to_str()?.to_string();
                let alias = alias.to_str()?.to_string();
                this.register_alias(command_name, alias);
                Ok(())
            }
        );
        methods.add_method_mut(
            "add_keybinding",
            |_, this, (key_seq, binding): (Table, mlua::String)| {
                let keys = key_seq.sequence_values()
                    .map(|k| k.map(|x: AnyUserData| x.take::<KeyPress>()))
                    .collect::<Result<Result<Vec<_>,_>, _>>()??;
                let binding = binding.to_str()?.to_string();
                this.add_keybinding(keys, binding);
                Ok(())
            }
        );
        methods.add_method(
            "get_keybinding",
            |_, this, (key_seq,): (Table,)| {
                let keys = key_seq.sequence_values()
                    .map(|k| k.map(|x: AnyUserData| x.take::<KeyPress>()))
                    .collect::<Result<Result<Vec<_>,_>, _>>()??;
                let binding = this.get_keybinding(&keys);
                Ok(binding)
            }
        )
    }
}