use std::collections::HashMap;
use mlua::{AnyUserData, Lua, ObjectLike, Table, UserDataMethods};
use mlua::prelude::{LuaTable, LuaUserData};
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

impl LuaUserData for MajorMode {
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
        methods.add_method(
            "modify_line",
            |_, _, (styled_file, _total_lines): (AnyUserData, mlua::Integer)| {
                Ok(styled_file)
            }
        );
    }
}


pub fn major_mode_module(lua: &Lua) -> mlua::Result<LuaTable> {
    let exports = lua.create_table()?;
    let metatable = lua.create_table()?;
    
    metatable.set(
        "__call",
        lua.create_function(|lua,(this_table,): (Table,)| {
            let user_data = lua.create_userdata(MajorMode::new())?;
            let proxy = lua.create_table()?;
            proxy.set("__userdata", user_data.clone())?;
            proxy.set("__class", this_table)?;

            let method_names = vec!["modify_line", "register_command", "register_alias"];

            for method_name in method_names {
                if let Ok(method) = user_data.get::<mlua::Function>(method_name) {
                    let ud_clone = user_data.clone();
                    let wrapped = lua.create_function(move |_, (_self, args): (mlua::Value, mlua::MultiValue)| {
                        // Ignore _self (the proxy table), use userdata instead
                        let mut call_args = mlua::MultiValue::new();
                        call_args.push_front(mlua::Value::UserData(ud_clone.clone()));
                        for arg in args {
                            call_args.push_back(arg);
                        }
                        method.call::<mlua::MultiValue>(call_args)
                    })?;
                    proxy.set(method_name, wrapped)?;
                }
            }
            
            let mt = lua.create_table()?;
            mt.set(
                "__index", 
                lua.create_function(|_, (table, key): (Table, mlua::Value)| {
                    if let Ok(class) = table.get::<Table>("__class") {
                        let fallback = class.get::<mlua::Value>(key)?;
                        if !matches!(fallback, mlua::Value::Nil) {
                            return Ok(fallback);
                        }
                    }

                    Ok(mlua::Value::Nil)
                })?
            )?;
            mt.set(
                "__newindex",
                lua.create_function(|_, (table, key, value): (Table, mlua::Value, mlua::Value)| {
                    table.raw_set(key, value)?;
                    Ok(())
                })?
            )?;
            
            proxy.set_metatable(Some(mt))?;
            
            Ok(proxy)
        })?
    )?;
    
    exports.set_metatable(Some(metatable))?;
    
    Ok(exports)
}