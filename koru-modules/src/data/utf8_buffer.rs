use std::sync::Arc;
use mlua::{Lua, UserData, UserDataFields, UserDataMethods, Value};
use mlua::prelude::{LuaResult, LuaTable};

#[derive(Default)]
pub struct Utf8Buffer {
    buffer: String,
}

impl Utf8Buffer {
    pub fn new(buffer: String) -> Self {
        Utf8Buffer { buffer }
    }
}

impl UserData for Utf8Buffer {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("push", |_, this, (lua_string,): (mlua::String,)| {
            this.buffer.push_str(&*lua_string.to_str()?);
            Ok(())
        });

        methods.add_method_mut("pop_n_char", |_, this, (size,): (mlua::Integer,)| {
            for _ in 0..size {
                this.buffer.pop();
            }
            Ok(())
        });
    }

    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_function_get("length", |_, this| {
            Ok(this.borrow::<Utf8Buffer>()?.buffer.len())
        });
        fields.add_field_function_get("to_string", |_, this| {
            Ok(this.borrow::<Utf8Buffer>()?.buffer.to_string())
        });
    }
}

pub fn utf8_buffer_mod(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    let utf8_metatable = lua.create_table()?;

    utf8_metatable.set(
        "__call",
        lua.create_function(|lua, args: mlua::MultiValue| {
            let (args, _vaargs) = args.as_slices();
            let user_data = match args {
                [Value::String(value)] => {
                    lua.create_userdata(Utf8Buffer::new(value.to_str()?.to_string()))?
                }
                [] => {
                    lua.create_userdata(Utf8Buffer::default())?
                }
                _ => return Err(mlua::Error::BadArgument {
                    cause: Arc::new(mlua::Error::external("invalid arguments to Utf8Buffer")),
                    to: None,
                    pos: 1,
                    name: None,
                })
            };
            Ok(user_data)
        })?
    )?;
    exports.set_metatable(Some(utf8_metatable))?;

    Ok(exports)
}