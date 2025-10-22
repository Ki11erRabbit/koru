use std::collections::HashMap;
use std::mem::ManuallyDrop;
use guile_rs::scheme_object::{SchemeObject, SchemeProcedure};
use guile_rs::{SmobData, SmobDrop, SmobEqual, SmobPrint, SmobSize};
use crate::kernel::scheme_api::Command;

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