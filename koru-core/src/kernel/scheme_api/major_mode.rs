mod text_view;
pub(crate) mod text_edit;

use scheme_rs::gc::Gc;
use std::sync::{Arc};
use scheme_rs::exceptions::Condition;
use scheme_rs::gc::Trace;
use scheme_rs::proc::Procedure;
use scheme_rs::records::{rtd, Record, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::value::Value;
#[derive(Clone, Debug, Trace)]
pub struct MajorMode {
    name: String,
    data: Value,
    draw: Procedure,
}

impl MajorMode {
    pub fn new(
        name: String,
        data: Value,
        draw: Procedure,
    ) -> Self {
        MajorMode {
            name,
            data,
            draw,
        }
    }

    pub fn draw(&self) -> Procedure {
        self.draw.clone()
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
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let name: String = name.clone().try_into()?;
    let Some((draw, rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let draw: Procedure = draw.clone().try_into()?;
    let data = if let Some((data, _)) = rest.split_first() {
        data.clone()
    } else {
        Value::undefined()
    };

    let major_mode = MajorMode::new(name, data, draw);

    Ok(vec![Value::from(Record::from_rust_type(major_mode))])
}

#[bridge(name = "major-mode-data", lib = "(major-mode)")]
pub fn major_mode_data(mode: &Value) -> Result<Vec<Value>, Condition> {
    let mode: Gc<MajorMode> = mode.clone().try_into_rust_type()?;
    Ok(vec![mode.data.clone()])
}


/*
#[bridge(name = "major-mode-append-line", lib = "(major-mode)")]
pub async fn append_line(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((mode, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let Some((current_line, rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let Some((total_lines, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let mode: Gc<MajorMode> = mode.clone().try_into_rust_type()?;

    let mod_line = mode.read().prepend_line.clone();

    if let Some(mod_line) = mod_line {
        let result = mod_line.call(&[current_line.clone(), total_lines.clone()]).await?;
        Ok(result)
    } else {
        Ok(vec![])
    }
}*/
/*
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
*/
#[bridge(name = "debug-print", lib = "(major-mode)")]
pub fn debug_print() -> Result<Vec<Value>, Condition> {
    println!("debug_print");
    Ok(vec![])
}
