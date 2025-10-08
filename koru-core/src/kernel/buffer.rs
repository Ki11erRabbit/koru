//! The Buffer is a window into the editor.
//!
//! The buffer provides an opaque space to allow for the editor to work with
//!

use mlua::{Lua, Table, UserData, UserDataMethods, Value};
use crate::kernel::modes::{KeyBuffer, MajorMode, MinorMode};


pub struct Buffer {
    major_mode: MajorMode,
    minor_modes: Vec<MinorMode>,
    key_buffer: Option<KeyBuffer>,
}

impl UserData for Buffer {

}


pub fn buffer_mod(lua: &Lua) -> mlua::Result<Table> {
    let exports = lua.create_table()?;
    let buffer_table = lua.create_table()?;

    let buffer_constructor = lua.create_function(|lua, args: mlua::MultiValue | {
        let (args, _) = args.as_slices();
        match args {
            [major_mode, minor_mode] => {
                let major_mode = major_mode.as_userdata()
                    .unwrap()
                    .borrow::<MajorMode>()?;
                let minor_mode = minor_mode.as_table()
                    .unwrap()
                    .sequence_values::<MinorMode>()
                    .collect()?;

                lua.create_userdata(Buffer { 
                    major_mode: (*major_mode).clone(), 
                    minor_modes ,
                    key_buffer: None,
                })
            }
            [major_mode, minor_mode, key_buffer] => {
                let major_mode = major_mode.as_userdata()
                    .unwrap()
                    .borrow::<MajorMode>()?;
                let minor_mode = minor_mode.as_table()
                    .unwrap()
                    .sequence_values::<MinorMode>()
                    .collect()?;
                
                lua.create_userdata(Buffer {
                    major_mode: (*major_mode).clone(),
                    minor_modes ,
                    key_buffer: None,
                })
            }
            _ => unimplemented!(),
        }
    })?;

    buffer_table.set(
        "__call",
        buffer_constructor
    )?;

    input_buffer_table.set(
        "__call",
        lua.create_function(|lua, args: mlua::MultiValue | {
            let buffer_base: Value = buffer_constructor.call(args)?;
            
        })?,
    )?;


    Ok(exports)
}