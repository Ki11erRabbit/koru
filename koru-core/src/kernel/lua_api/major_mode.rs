use std::collections::HashMap;
use mlua::{AnyUserData, Lua, UserData, UserDataMethods};
use mlua::prelude::LuaTable;
use crate::kernel::lua_api::Command;

pub struct MajorMode {
    commands: Vec<Command>,
    aliases: HashMap<String, usize>
}

impl MajorMode {
    pub fn new() -> Self {
        MajorMode {
            commands: Vec::new(),
            aliases: HashMap::new()
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
}

impl UserData for MajorMode {
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
    }
}


pub fn major_mode_module(lua: &Lua) -> mlua::Result<LuaTable> {
    let exports = lua.create_table()?;
    let metatable = lua.create_table()?;
    
    metatable.set(
        "__call",
        lua.create_function(|lua, _: ()| {
            lua.create_userdata(MajorMode::new())
        })?
    )?;
    
    Ok(exports)
}