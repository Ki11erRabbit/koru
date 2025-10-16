mod major_mode;
mod minor_mode;
mod ui_state;

use std::collections::HashMap;
use mlua::{AnyUserData, Function, Lua, MultiValue, Table, UserData, UserDataMethods, Value};
use mlua::prelude::LuaTable;
use std::borrow::Borrow;
use crate::styled_text::styled_text_module;

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
            async |_, this, _: ()| {
                let this = this.borrow();
                let value: () = this.function.call_async(()).await?;
                Ok(value)
            }
        );
        methods.add_method(
            "name",
            |_, this, _: ()| {
                Ok(this.name.clone())
            }
        );
        methods.add_method(
            "description",
            |_, this, _: ()| {
                Ok(this.description.clone())
            }
        );
        methods.add_method(
            "argument_description",
            |_, this, _: ()| {
                let list = this.arguments
                    .iter().map(|arg| {
                    <&ArgumentDef as Into<&str>>::into(arg).to_string()
                }).collect::<Vec<String>>();

                Ok(list)
            }
        );
    }
}


pub fn kernel_mod(lua: &Lua) -> mlua::Result<LuaTable> {
    let exports = lua.create_table()?;

    let package = lua.globals().get::<Table>("package")?;
    let preload = package.get::<Table>("preload")?;

    preload.set(
        "Koru.StyledText",
        lua.create_function(|lua, _: mlua::String| {
            styled_text_module(lua)
        })?
    )?;
    preload.set(
        "Koru.Command",
        lua.create_function(|lua, _: mlua::String| {
            let command_module = lua.create_table()?;
            let command_metatable = lua.create_table()?;

            command_metatable.set(
                "__call",
                lua.create_function(|lua, args: MultiValue| {
                    let (pos, _) = args.as_slices();
                    let command = match pos {
                        [_, Value::String(name), Value::String(description),
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
                        x => todo!("Handle invalid call properly: {x:?}")
                    };
                    lua.create_userdata(command)
                })?
            )?;

            command_module.set_metatable(Some(command_metatable))?;
            Ok(command_module)
        })?
    )?;
    preload.set(
        "Koru.MajorMode",
        lua.create_function(|lua, _: mlua::String| {
            let table = major_mode::major_mode_module(lua)?;
            Ok(table)
        })?
    )?;
    
    exports.set(
        "hello",
        lua.create_function(|_, _: ()| {
            println!("hello");
            Ok(())
        })?
    )?;

    Ok(exports)
}