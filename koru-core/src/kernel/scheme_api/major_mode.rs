mod text_view;
pub(crate) mod text_edit;

use scheme_rs::gc::Gc;
use std::sync::{Arc};
use scheme_rs::exceptions::{Condition, Exception};
use scheme_rs::gc::Trace;
use scheme_rs::proc::Procedure;
use scheme_rs::records::{rtd, Record, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::value::Value;
use tokio::sync::RwLock;
use crate::kernel::buffer::Cursor;

#[derive(Debug, Trace)]
pub struct MajorMode {
    name: String,
    data: RwLock<Value>,
    draw: Procedure,
    get_main_cursor: Procedure,
    gain_focus: Procedure,
    lose_focus: Procedure,
}

impl MajorMode {
    pub fn new(
        name: String,
        data: Value,
        draw: Procedure,
        get_main_cursor: Procedure,
        gain_focus: Procedure,
        lose_focus: Procedure,
    ) -> Self {
        MajorMode {
            name,
            data: RwLock::new(data),
            draw,
            get_main_cursor,
            gain_focus,
            lose_focus,
        }
    }

    pub fn draw(&self) -> Procedure {
        self.draw.clone()
    }
    
    pub fn gain_focus(&self) -> Procedure {
        self.gain_focus.clone()
    }
    
    pub fn lose_focus(&self) -> Procedure {
        self.lose_focus.clone()
    }
    
    pub async fn get_main_cursor(&self, self_value: Value) -> Result<Cursor, Exception> {
        let cursor: Gc<Cursor> = self.get_main_cursor.call(&[self_value]).await
            .map(|values| {
                let cursor = values[0].clone();
                cursor.try_into_rust_type()
        })??;
        Ok((*cursor).clone())
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
        return Err(Condition::wrong_num_of_args(5, args.len()));
    };
    let name: String = name.clone().try_into()?;
    let Some((draw, rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(5, args.len()));
    };
    let Some((get_main_cursor, rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(5, args.len()));
    };
    let Some((gain_focus, rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(5, args.len()));
    };
    let Some((lose_focus, rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(5, args.len()));
    };
    let get_main_cursor: Procedure = get_main_cursor.clone().try_into()?;
    let gain_focus: Procedure = gain_focus.clone().try_into()?;
    let lose_focus: Procedure = lose_focus.clone().try_into()?;
    let draw: Procedure = draw.clone().try_into()?;
    let data = if let Some((data, _)) = rest.split_first() {
        data.clone()
    } else {
        Value::undefined()
    };

    let major_mode = MajorMode::new(name, data, draw, get_main_cursor, gain_focus, lose_focus);

    Ok(vec![Value::from(Record::from_rust_type(major_mode))])
}

#[bridge(name = "major-mode-data", lib = "(major-mode)")]
pub async fn major_mode_data(mode: &Value) -> Result<Vec<Value>, Condition> {
    let mode: Gc<MajorMode> = mode.clone().try_into_rust_type()?;
    Ok(vec![mode.data.read().await.clone()])
}

#[bridge(name = "major-mode-data-set!", lib = "(major-mode)")]
pub async fn major_mode_set_data(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((mode, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };
    let Some((data_value, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };
    let mode: Gc<MajorMode> = mode.clone().try_into_rust_type()?;
    *mode.data.write().await = data_value.clone();
    Ok(Vec::new())
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
