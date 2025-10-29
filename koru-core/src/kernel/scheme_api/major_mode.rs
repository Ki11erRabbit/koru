use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::sync::LazyLock;
use guile_rs::scheme_object::{SchemeObject, SchemeProcedure, SchemeSmob};
use guile_rs::{Guile, Module, SchemeValue, SmobTag, SmobData, guile_wrong_type_arg};
use crate::kernel::scheme_api::command::{Command, COMMAND_SMOB_TAG};

pub static MAJOR_MODE_SMOB_TAG: LazyLock<SmobTag<MajorMode>> = LazyLock::new(|| {
    SmobTag::register("MajorMode") 
});

#[derive(Clone)]
struct MajorModeInternal {
    
}

#[derive(Clone)]
pub struct MajorMode {
    name: String,
    commands: Vec<Command>,
    aliases: HashMap<String, usize>,
    data: SchemeObject,
    modify_line: SchemeProcedure,
}

impl MajorMode {
    pub fn new(name: String, data: SchemeObject, modify_line: SchemeProcedure) -> Self {
        MajorMode {
            name,
            commands: Vec::new(),
            aliases: HashMap::new(),
            data,
            modify_line,
        }
    }
    
    pub fn register_command(&mut self, name: String, command: Command) {
        let index = self.commands.len();
        self.commands.push(command);
        self.aliases.insert(name, index);
    }
    
    pub fn register_alias(&mut self, name: String, alias: String) {
        let index = if let Some(index) = self.aliases.get(&name) {
            Some(*index)
        } else {
            None
        };
        if let Some(index) = index {
            self.aliases.insert(alias, index);
        }
    }
    
    pub fn remove_alias(&mut self, name: String) {
        self.aliases.remove(&name);
    }
}

impl SmobData for MajorMode {
    fn print(&self) -> String {
        format!("#<MajorMode:{}>", self.name)
    }
    fn heap_size(&self) -> usize {
        let command_mem_size = self.commands.capacity() * size_of::<Command>();
        let string_size = self.name.capacity();
        let aliases_size = self.aliases.capacity() * (size_of::<String>() + size_of::<usize>());

        command_mem_size + string_size + aliases_size
    }
}


pub extern "C" fn major_mode_create(name: SchemeValue, modify_line: SchemeValue, data: SchemeValue) -> SchemeValue {
    let Some(name) = SchemeObject::from(name).cast_symbol() else {
        guile_wrong_type_arg!("major-mode-create", 1, name);
    };
    let Some(modify_line) = SchemeObject::from(modify_line).cast_procedure() else {
        guile_wrong_type_arg!("major-mode-create", 2, modify_line);
    };
    let data = SchemeObject::from(data);
    
    let major_mode = MajorMode::new(name.to_string(), data, modify_line);
    
    let smob = MAJOR_MODE_SMOB_TAG.make(major_mode);
    
    <SchemeSmob<_> as Into<SchemeObject>>::into(smob).into()
}

extern "C" fn major_mode_data(mode: SchemeValue) -> SchemeValue {
    let Some(mode) = SchemeObject::from(mode).cast_smob(MAJOR_MODE_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("major-mode-data", 1, mode);
    };
    
    mode.borrow().data.clone().into()
}

pub extern "C" fn major_mode_register_command(mode: SchemeValue, command: SchemeValue) -> SchemeValue {
    let Some(mode) = SchemeObject::from(mode).cast_smob(MAJOR_MODE_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("major-mode-register-command", 1, mode);
    };
    let Some(command) = SchemeObject::from(command).cast_smob(COMMAND_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("major-mode-register-command", 2, command);
    };
    
    mode.borrow_mut().register_command(command.borrow().name().to_string(), (command.borrow()).clone());
    
    SchemeValue::undefined()
}

pub extern "C" fn major_mode_register_alias(mode: SchemeValue, command_name: SchemeValue, alias: SchemeValue) -> SchemeValue {
    let Some(mode) = SchemeObject::from(mode).cast_smob(MAJOR_MODE_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("major-mode-register-alias", 1, mode);
    };
    let Some(command_name) = SchemeObject::from(command_name).cast_symbol() else {
        guile_wrong_type_arg!("major-mode-register-alias", 2, command_name);
    };
    let Some(alias) = SchemeObject::from(alias).cast_symbol() else {
        guile_wrong_type_arg!("major-mode-register-alias", 2, command_name);
    };
    
    mode.borrow_mut().register_alias(command_name.to_string(), alias.to_string());
    
    SchemeValue::undefined()
}

pub extern "C" fn major_mode_unregister(mode: SchemeValue, command_name: SchemeValue) -> SchemeValue {
    let Some(mode) = SchemeObject::from(mode).cast_smob(MAJOR_MODE_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("major-mode-unregister-alias", 1, mode);
    };
    let Some(command_name) = SchemeObject::from(command_name).cast_symbol() else {
        guile_wrong_type_arg!("major-mode-unregister-alias", 2, command_name);
    };
    
    mode.borrow_mut().remove_alias(command_name.to_string());
    SchemeValue::undefined()
}

pub extern "C" fn major_mode_modify_line(mode: SchemeValue, styled_file: SchemeValue, total_lines: SchemeValue) -> SchemeValue {
    let Some(mode) = SchemeObject::from(mode).cast_smob(MAJOR_MODE_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("major-mode-modify-line", 1, mode);
    };
    let styled_file = mode.borrow().modify_line.call2(SchemeObject::from(styled_file), SchemeObject::from(total_lines));
    
    styled_file.into()
}

pub extern "C" fn modify_line_default(styled_file: SchemeValue, _total_lines: SchemeValue) -> SchemeValue {
    styled_file
}

pub fn major_mode_module() {
    Guile::define_fn("major-mode-create", 2, 1, false, 
        major_mode_create as extern "C" fn(SchemeValue, SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("major-mode-data", 1, 0, false,
        major_mode_data as extern "C" fn(SchemeValue) -> SchemeValue
    );
    Guile::define_fn("major-mode-register-command", 2, 0, false, 
        major_mode_register_command as extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("major-mode-register-alias", 3, 0, false,
        major_mode_register_alias as extern "C" fn(SchemeValue, SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("major-mode-unregister-alias", 2, 0, false,
        major_mode_unregister as extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("major-mode-modify-line", 3, 0, false,
        major_mode_modify_line as extern "C" fn(SchemeValue, SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("modify-line-default", 2, 0, false,
        modify_line_default as extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue
    );
    
    let mut module = Module::new("major-mode", Box::new(|x: &mut ()| {}));
    module.add_export("major-mode-create");
    module.add_export("major-mode-data");
    module.add_export("major-mode-register-command");
    module.add_export("major-mode-register-alias");
    module.add_export("major-mode-unregister-alias");
    module.add_export("major-mode-modify-line");
    module.add_export("modify-line-default");
    module.export();
    module.define(&mut ());
}