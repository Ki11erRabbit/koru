
use mlua::{UserData, UserDataMethods};

pub struct MajorMode;

impl MajorMode {
}


impl UserData for MajorMode {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        
    }
}