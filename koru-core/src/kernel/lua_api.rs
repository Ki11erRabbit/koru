use std::collections::HashMap;
use mlua::{AnyUserData, Function, Lua, MultiValue, UserData, UserDataMethods, Value};
use mlua::prelude::LuaTable;
use std::borrow::Borrow;


pub enum ArgumentDef {
    Text,
    Number,
    Path,
    Variable(Box<ArgumentDef>),
}

impl From<ArgumentDef> for &str {
    fn from(def: ArgumentDef) -> Self {
        match def {
            ArgumentDef::Text => "text",
            ArgumentDef::Number => "number",
            ArgumentDef::Path => "path",
            ArgumentDef::Variable(x) => {
                match x.as_ref() {
                    ArgumentDef::Text => "variable:text",
                    ArgumentDef::Number => "variable:number",
                    ArgumentDef::Path => "variable:path",
                    _ => unreachable!("invalid variable arg")
                }
            }
        }
    }
}

impl From<&ArgumentDef> for &str {
    fn from(def: &ArgumentDef) -> Self {
        match def {
            ArgumentDef::Text => "text",
            ArgumentDef::Number => "number",
            ArgumentDef::Path => "path",
            ArgumentDef::Variable(x) => {
                match x.as_ref() {
                    ArgumentDef::Text => "variable:text",
                    ArgumentDef::Number => "variable:number",
                    ArgumentDef::Path => "variable:path",
                    _ => unreachable!("invalid variable arg")
                }
            }
        }
    }
}
impl TryFrom<&str> for ArgumentDef {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "text" => Ok(ArgumentDef::Text),
            "number" => Ok(ArgumentDef::Number),
            "path" => Ok(ArgumentDef::Path),
            "variable:text" => Ok(ArgumentDef::Variable(Box::new(ArgumentDef::Text))),
            "variable:number" => Ok(ArgumentDef::Variable(Box::new(ArgumentDef::Number))),
            "variable:path" => Ok(ArgumentDef::Variable(Box::new(ArgumentDef::Path))),
            _ => {
                Err(format!("Unknown argument: {}", value))
            },
        }
    }
}

impl TryFrom<String> for ArgumentDef {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        ArgumentDef::try_from(value.as_str())
    }
}

pub struct Command {
    name: String,
    function: Function,
    description: String,
    arguments: Vec<ArgumentDef>
}

impl UserData for Command {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method(
            "apply",
            async |lua, this, _: ()| {
                let this = this.borrow();
                let value: () = this.function.call_async(()).await?;
                Ok(value)
            }
        );
        methods.add_method(
            "name",
            |_, this, _: ()| {
                let this = this.borrow();
                Ok(this.name.clone())
            }
        );
        methods.add_method(
            "description",
            |_, this, _: ()| {
                let this = this.borrow();
                Ok(this.description.clone())
            }
        );
        methods.add_method(
            "argument_description",
            |_, this, _: ()| {
                let this = this.borrow();
                let list = this.arguments
                    .iter().map(|arg| {
                    <&ArgumentDef as Into<&str>>::into(arg).to_string()
                }).collect::<Vec<String>>();

                Ok(list)
            }
        );
    }
}

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
            |lua, this, (arg,): (AnyUserData,)| {
                let command = arg.take::<Command>()?;
                this.register_command(command.name.clone(), command);
                Ok(())
            }
        );
        methods.add_method_mut(
            "register_alias",
            |lua, this, (command_name, alias): (mlua::String, mlua::String)| {
                let command_name = command_name.to_str()?.to_string();
                let alias = alias.to_str()?.to_string();
                this.register_alias(command_name, alias);
                Ok(())
            }
        );
    }
}


pub fn kernel_mod(lua: &Lua) -> mlua::Result<LuaTable> {
    let exports = lua.create_table()?;
    
    let command_module = lua.create_table()?;
    let command_metatable = lua.create_table()?;

    command_metatable.set(
        "__call",
        lua.create_function(|lua, args: MultiValue| {
            let (pos, _) = args.as_slices();
            let command = match pos {
                [Value::String(name), Value::String(description),
                Value::Function(fun), Value::Table(table)] => {
                    let arguments = table.sequence_values::<mlua::String>()
                        .map(|x| {
                        x.map(|x| {
                            x.to_str().map(|x| {
                                TryFrom::try_from(x.to_string().as_str())
                            })
                        })
                    }).collect::<Result<Result<Result<Vec<_>, _>, _>, _>>()??.unwrap();

                    Command {
                        name: name.to_str()?.to_string(),
                        description: description.to_str()?.to_string(),
                        function: fun.clone(),
                        arguments
                    }
                }
                _ => return Err(mlua::Error::runtime("Invalid call"))
            };
            Ok(command)
        })?
    )?;
    
    command_module.set_metatable(Some(command_metatable))?;
    
    exports.set(
        "Command",
        command_module
    )?;


    /*exports.set(
        "open_file",
        lua.create_async_function(async |lua, path: String| {
            let handle = open_or_get_handle(path).await.unwrap();
            lua.create_userdata(handle)
        })?,
    )?;*/

    let package = exports.get::<mlua::Table>("package")?;
    let preload = package.get::<mlua::Table>("preload")?;
    /*preload.set(
        "Key",
        lua.create_function(|lua, ()| {
            key::key_module(lua)
        })?
    )?;*/
    

    Ok(exports)
}