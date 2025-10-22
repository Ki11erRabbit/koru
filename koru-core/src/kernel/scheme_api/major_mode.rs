use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::sync::LazyLock;
use guile_rs::scheme_object::{SchemeObject, SchemeProcedure};
use guile_rs::{SchemeValue, Smob, SmobData, SmobDrop, SmobEqual, SmobPrint, SmobSize};
use crate::kernel::scheme_api::{Command, COMMAND_SMOB};

pub static MAJOR_MODE_SMOB_TAG: LazyLock<Smob<MajorMode>> = LazyLock::new(|| {
    Smob::register("MajorMode") 
});

#[derive(Clone)]
struct MajorModeInternal {
    name: String,
    commands: Vec<Command>,
    aliases: HashMap<String, usize>,
    data: SchemeObject,
    modify_line: SchemeProcedure,
}

#[derive(Clone)]
pub struct MajorMode {
    internal: ManuallyDrop<MajorModeInternal>,
}

impl MajorMode {
    pub fn new(name: String, data: SchemeObject, modify_line: SchemeProcedure) -> Self {
        MajorMode {
            internal: ManuallyDrop::new(MajorModeInternal {
                name,
                commands: Vec::new(),
                aliases: HashMap::new(),
                data,
                modify_line,
            })
        }
    }
    
    pub fn register_command(&mut self, name: String, command: Command) {
        let index = self.internal.commands.len();
        self.internal.commands.push(command);
        self.internal.aliases.insert(name, index);
    }
    
    pub fn register_alias(&mut self, name: String, alias: String) {
        let index = if let Some(index) = self.internal.aliases.get(&name) {
            Some(*index)
        } else {
            None
        };
        if let Some(index) = index {
            self.internal.aliases.insert(alias, index);
        }
    }
    
    pub fn remove_alias(&mut self, name: String) {
        self.internal.aliases.remove(&name);
    }
}

impl SmobPrint for MajorMode {
    fn print(&self) -> String {
        format!("#<MajorMode:{}>", self.internal.name)
    }
}

impl SmobDrop for MajorMode {
    fn drop(&mut self) -> usize {
        let command_mem_size = self.internal.commands.capacity() * std::mem::size_of::<Command>();
        let string_size = self.internal.name.capacity();
        let aliases_size = self.internal.aliases.capacity() * (std::mem::size_of::<String>() + std::mem::size_of::<usize>());
        
        unsafe {
            ManuallyDrop::drop(&mut self.internal);
        }
        
        command_mem_size + string_size + aliases_size
    }

    fn heap_size(&self) -> usize {
        let command_mem_size = self.internal.commands.capacity() * std::mem::size_of::<Command>();
        let string_size = self.internal.name.capacity();
        let aliases_size = self.internal.aliases.capacity() * (std::mem::size_of::<String>() + std::mem::size_of::<usize>());

        command_mem_size + string_size + aliases_size
    }
}

impl SmobEqual for MajorMode {}

impl SmobSize for MajorMode {}

impl SmobData for MajorMode {}

pub extern "C" fn major_mode_create(name: SchemeValue, modify_line: SchemeValue, data: SchemeValue) -> SchemeValue {
    let Some(name) = SchemeObject::new(name).cast_string() else {
        return SchemeObject::undefined().into()
    };
    let Some(modify_line) = SchemeObject::new(modify_line).cast_procedure() else {
        return SchemeObject::undefined().into()
    };
    let data = SchemeObject::new(data);
    
    let major_mode = MajorMode::new(name.to_string(), data, modify_line);
    
    let smob = MAJOR_MODE_SMOB_TAG.make(major_mode);
    
    smob.into()
}

pub extern "C" fn major_mode_register_command(mode: SchemeValue, command: SchemeValue) -> SchemeValue {
    let Some(mode) = SchemeObject::new(mode).cast_smob(MAJOR_MODE_SMOB_TAG.clone()) else {
        return SchemeObject::undefined().into() 
    };
    let Some(command) = SchemeObject::new(command).cast_smob(COMMAND_SMOB.clone()) else {
        return SchemeObject::undefined().into()
    };
    
    mode.register_command(command.name().to_string(), command);
    
    SchemeObject::undefined().into()
}

pub extern "C" fn major_mode_register_alias(mode: SchemeValue, command_name: SchemeValue, alias: SchemeValue) -> SchemeValue {
    let Some(mode) = SchemeObject::new(mode).cast_smob(MAJOR_MODE_SMOB_TAG.clone()) else {
        return SchemeObject::undefined().into()
    };
    let Some(command_name) = SchemeObject::new(command_name).cast_string() else {
        return SchemeObject::undefined().into()
    };
    let Some(alias) = SchemeObject::new(alias).cast_string() else {
        return SchemeObject::undefined().into()
    };
    
    mode.register_alias(command_name.to_string(), alias.to_string());
    
    SchemeObject::undefined().into()
}

pub extern "C" fn major_mode_unregister(mode: SchemeValue, command_name: SchemeValue) {
    let Some(mode) = SchemeObject::new(mode).cast_smob(MAJOR_MODE_SMOB_TAG.clone()) else {
        return SchemeObject::undefined().into()
    };
    let Some(command_name) = SchemeObject::new(command_name).cast_string() else {
        return SchemeObject::undefined().into()
    };
    
    mode.remove_alias(command_name.to_string());
}

pub extern "C" fn major_mode_modify_line(mode: SchemeValue, styled_file: SchemeValue, total_lines: SchemeValue) -> SchemeValue {
    let Some(mode) = SchemeObject::new(mode).cast_smob(MAJOR_MODE_SMOB_TAG.clone()) else {
        return SchemeObject::undefined().into()
    };
}