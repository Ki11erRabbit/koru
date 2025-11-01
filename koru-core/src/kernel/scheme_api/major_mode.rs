use scheme_rs::gc::Gc;
use std::collections::HashMap;
use std::sync::{Arc};
use scheme_rs::exceptions::Condition;
use scheme_rs::gc::Trace;
use scheme_rs::proc::Procedure;
use scheme_rs::records::{rtd, Record, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::value::Value;
use crate::kernel::scheme_api::command::{Command};


#[derive(Clone, Debug, Trace)]
pub struct MajorMode {
    name: String,
    commands: Vec<Gc<Command>>,
    aliases: HashMap<String, usize>,
    data: Value,
    modify_line: Procedure,
}

impl MajorMode {
    pub fn new(name: String, data: Value, modify_line: Procedure) -> Self {
        MajorMode {
            name,
            commands: Vec::new(),
            aliases: HashMap::new(),
            data,
            modify_line,
        }
    }
    
    pub fn register_command(&mut self, name: String, command: Gc<Command>) {
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

impl SchemeCompatible for MajorMode {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&MajorMode")
    }
}

#[bridge(name = "major-mode-create", lib = "(major-mode)")]
pub fn major_mode_create(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((name, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };
    let name: String = name.clone().try_into()?;
    let Some((modify_line, rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };
    let modify_line: Procedure = modify_line.clone().try_into()?;
    let data = if let Some((data, _)) = rest.split_first() {
        data.clone()
    } else {
        Value::undefined()
    };

    let major_mode = MajorMode::new(name, data, modify_line);

    Ok(vec![Value::from(Record::from_rust_type(major_mode))])
}

#[bridge(name = "major-mode-data", lib = "(major-mode)")]
pub fn major_mode_data(mode: &Value) -> Result<Vec<Value>, Condition> {
    let mode: Gc<MajorMode> = mode.clone().try_into_rust_type()?;
    Ok(vec![mode.read().clone().data])
}

#[bridge(name = "major-mode-register-command", lib = "(major-mode)")]
pub fn major_mode_register_command(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((mode, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };
    let Some((command, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };
    let mode: Gc<MajorMode> = mode.clone().try_into_rust_type()?;
    let command: Gc<Command> = command.clone().try_into_rust_type()?;

    mode.write().register_command(command.read().name().to_string(), command.clone());

    Ok(Vec::new())
}



/*pub extern "C" fn major_mode_modify_line(mode: SchemeValue, styled_file: SchemeValue, total_lines: SchemeValue) -> SchemeValue {
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
}*/