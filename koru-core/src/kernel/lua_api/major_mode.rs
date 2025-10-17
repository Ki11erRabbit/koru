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
        lua.create_function(|lua, (this_table,): (Table,)| {
            let user_data = lua.create_userdata(MajorMode::new())?;
            let proxy = lua.create_table()?;
            proxy.set("__userdata", user_data.clone())?;
            proxy.set("__class", this_table)?;

            let mt = lua.create_table()?;
            let ud_for_index = user_data.clone();

            mt.set(
                "__index",
                lua.create_function(move |lua, (table, key): (Table, mlua::String)| {
                    // Check __class for derived/override methods
                    if let Ok(class) = table.get::<Table>("__class") {
                        if let Ok(value) = class.get::<mlua::Value>(key.clone()) {
                            if !matches!(value, mlua::Value::Nil) {
                                return Ok(value);
                            }
                        }
                    }

                    // Then check userdata and wrap the method
                    if let Ok(method) = ud_for_index.get::<mlua::Function>(key.clone()) {
                        let ud_clone = ud_for_index.clone();
                        return Ok(mlua::Value::Function(
                            lua.create_function(move |_, (_self, args): (mlua::Value, mlua::MultiValue)| {
                                let mut call_args = mlua::MultiValue::new();
                                call_args.push_front(mlua::Value::UserData(ud_clone.clone()));
                                for arg in args {
                                    call_args.push_back(arg);
                                }
                                method.call::<mlua::MultiValue>(call_args)
                            })?
                        ));
                    }

                    Ok(mlua::Value::Nil)
                })?
            )?;

            mt.set(
                "__newindex",
                lua.create_function(move |_, (table, key, value): (Table, mlua::Value, mlua::Value)| {
                    table.raw_set(key, value)?;
                    Ok(())
                })?
            )?;

            proxy.set_metatable(Some(mt))?;

            Ok(proxy)
        })?
    )?;

    // Add an extend/inherit helper function
    exports.set(
        "extend",
        lua.create_function(|lua, (base_instance, derived_class): (Table, Table)| {
            // Get the Rust metatable using Lua's getmetatable
            let getmetatable: mlua::Function = lua.globals().get("getmetatable")?;
            let rust_mt: Table = getmetatable.call(base_instance.clone())?;

            let rust_index = rust_mt.get::<mlua::Function>("__index")?;
            let rust_newindex = rust_mt.get::<mlua::Function>("__newindex")?;

            // Create new metatable that chains to derived class
            let new_mt = lua.create_table()?;

            let derived_clone = derived_class.clone();
            let index_fn = lua.create_function(move |lua, (table, key): (Table, mlua::Value)| {
                // Check derived class first
                if let Ok(value) = derived_clone.get::<mlua::Value>(key.clone()) {
                    if !matches!(value, mlua::Value::Nil) {
                        return Ok(value);
                    }
                }

                // Fall back to Rust index
                rust_index.call::<mlua::Value>((table, key))
            })?;

            new_mt.set("__index", index_fn)?;
            new_mt.set("__newindex", rust_newindex)?;

            // Use Lua's setmetatable
            let setmetatable: mlua::Function = lua.globals().get("setmetatable")?;
            setmetatable.call::<()>((base_instance.clone(), new_mt))?;

            Ok(base_instance)
        })?
    )?;

    exports.set_metatable(Some(metatable))?;

    Ok(exports)
}