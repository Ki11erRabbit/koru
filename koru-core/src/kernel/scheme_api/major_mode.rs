use scheme_rs::gc::Gc;
use std::collections::HashMap;
use std::sync::{Arc};
use scheme_rs::exceptions::Condition;
use scheme_rs::gc::Trace;
use scheme_rs::num::Number;
use scheme_rs::proc::Procedure;
use scheme_rs::records::{rtd, Record, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::value::Value;
use crate::kernel::scheme_api::command::{Command};
use crate::styled_text::StyledFile;

#[derive(Clone, Debug, Trace)]
pub struct MajorMode {
    name: String,
    commands: Vec<Gc<Command>>,
    aliases: HashMap<String, usize>,
    data: Value,
    modify_line: Option<Procedure>,
}

impl MajorMode {
    pub fn new(name: String, data: Value, modify_line: Option<Procedure>) -> Self {
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

impl Default for MajorMode {
    fn default() -> Self {
        MajorMode::new(String::from("Bogus"), Value::from(false), None)
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

    let major_mode = MajorMode::new(name, data, Some(modify_line));

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

#[bridge(name = "major-mode-modify-lines", lib = "(major-mode)")]
pub async fn modify_line(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((mode, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };
    let Some((styled_file, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };
    let mode: Gc<MajorMode> = mode.clone().try_into_rust_type()?;
    let styled_file_core: Gc<StyledFile> = styled_file.clone().try_into_rust_type()?;

    let mod_line = mode.read().modify_line.clone();

    if let Some(mod_line) = mod_line {
        let len = styled_file_core.read().line_count();
        
        mod_line.call(&[styled_file.clone(), Value::from(Number::FixedInteger(len as i64))]).await?;
        Ok(vec![styled_file.clone()])
    } else {
        Ok(vec![styled_file.clone()])
    }
}

#[bridge(name = "write-line-number", lib = "(major-mode)")]
pub fn write_line_number(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((current_line, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let Some((max_lines, rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let Some((separator, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let current_line: Arc<Number> = current_line.clone().try_into()?;
    let current_line = current_line.checked_add(&Number::FixedInteger(1)).unwrap();
    let max_lines: Arc<Number> = max_lines.clone().try_into()?;
    let separator: char = separator.clone().try_into()?;
    let current_line = current_line.to_string();
    let max_lines = max_lines.to_string();
    let needed_padding = max_lines.chars().count();
    
    let mut string = format!("{: >1$}", current_line, needed_padding);
    string.push(separator);
    
    Ok(vec![Value::from(string)])
}

#[bridge(name = "modify-lines-default", lib = "(major-mode)")]
pub fn modify_line_default(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((file, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };
    let Some((_total_lines, _rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };

    Ok(vec![file.clone()])
}

