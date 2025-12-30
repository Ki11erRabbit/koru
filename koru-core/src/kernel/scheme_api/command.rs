use std::mem::ManuallyDrop;
use std::sync::{Arc, LazyLock};
use scheme_rs::exceptions::Condition;
use scheme_rs::gc::{Gc, Trace};
use scheme_rs::proc::Procedure;
use scheme_rs::records::{rtd, Record, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::value::Value;

#[derive(Clone, Debug, Trace)]
pub enum ArgumentDef {
    Text,
    Number,
    Path,
    KeyPress,
    KeySequence,
    Boolean,
    Variable(Box<ArgumentDef>),
}

impl From<ArgumentDef> for &str {
    fn from(def: ArgumentDef) -> Self {
        match def {
            ArgumentDef::Text => "text",
            ArgumentDef::Number => "number",
            ArgumentDef::Path => "path",
            ArgumentDef::KeyPress => "key-press",
            ArgumentDef::KeySequence => "key-sequence",
            ArgumentDef::Boolean => "boolean",
            ArgumentDef::Variable(x) => {
                match x.as_ref() {
                    ArgumentDef::Text => "variable:text",
                    ArgumentDef::Number => "variable:number",
                    ArgumentDef::Path => "variable:path",
                    ArgumentDef::KeyPress => "variable:key-press",
                    ArgumentDef::KeySequence => "variable:key-sequence",
                    ArgumentDef::Boolean => "variable:boolean",
                    _ => unreachable!("invalid variable arg")
                }
            }
        }
    }
}

impl From<&ArgumentDef> for &str {
    fn from(def: &ArgumentDef) -> Self {
        match def {
            ArgumentDef::Text => "text",
            ArgumentDef::Number => "number",
            ArgumentDef::Path => "path",
            ArgumentDef::KeyPress => "key-press",
            ArgumentDef::KeySequence => "key-sequence",
            ArgumentDef::Boolean => "boolean",
            ArgumentDef::Variable(x) => {
                match x.as_ref() {
                    ArgumentDef::Text => "variable:text",
                    ArgumentDef::Number => "variable:number",
                    ArgumentDef::Path => "variable:path",
                    ArgumentDef::KeyPress => "variable:key-press",
                    ArgumentDef::KeySequence => "variable:key-sequence",
                    ArgumentDef::Boolean => "variable:boolean",
                    _ => unreachable!("invalid variable arg")
                }
            }
        }
    }
}
impl TryFrom<&str> for ArgumentDef {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "text" => Ok(ArgumentDef::Text),
            "number" => Ok(ArgumentDef::Number),
            "path" => Ok(ArgumentDef::Path),
            "key-press" => Ok(ArgumentDef::KeyPress),
            "key-sequence" => Ok(ArgumentDef::KeySequence),
            "boolean" => Ok(ArgumentDef::Boolean),
            "variable:text" => Ok(ArgumentDef::Variable(Box::new(ArgumentDef::Text))),
            "variable:number" => Ok(ArgumentDef::Variable(Box::new(ArgumentDef::Number))),
            "variable:path" => Ok(ArgumentDef::Variable(Box::new(ArgumentDef::Path))),
            "variable:key-press" => Ok(ArgumentDef::Variable(Box::new(ArgumentDef::KeyPress))),
            "variable:key-sequence" => Ok(ArgumentDef::Variable(Box::new(ArgumentDef::KeySequence))),
            "variable:boolean" => Ok(ArgumentDef::Variable(Box::new(ArgumentDef::Boolean))),
            _ => {
                Err(format!("Unknown argument: {}", value))
            },
        }
    }
}

impl TryFrom<String> for ArgumentDef {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        ArgumentDef::try_from(value.as_str())
    }
}

impl SchemeCompatible for ArgumentDef {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&ArgumentDef")
    }
}



#[derive(Clone, Debug, Trace)]
pub struct Command {
    name: String,
    function: Procedure,
    description: String,
    arguments: Vec<ArgumentDef>,
}

impl Command {

    pub fn new(name: String, function: Procedure, description: String, arguments: Vec<ArgumentDef>) -> Self {
        Command {
            name,
            function,
            description,
            arguments,
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    
    pub fn command(&self) -> &Procedure {
        &self.function
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Command {}


impl SchemeCompatible for Command {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&Command")
    }
}

#[bridge(name = "=command", lib = "(koru-command)")]
pub fn equal_command(args: &[Value]) -> Result<Vec<Value>, Condition> {
    if let Some((first, rest)) = args.split_first() {
        let first: Gc<Command> = first.clone().try_into_rust_type()?;
        for next in rest {
            let next: Gc<Command> = next.clone().try_into_rust_type()?;
            if first != next {
                return Ok(vec![Value::from(false)]);
            }
        }
    }
    Ok(vec![Value::from(true)])
}

#[bridge(name = "command-apply", lib = "(koru-command)")]
pub async fn command_apply(args: &[Value]) -> Result<Vec<Value>, Condition> {
    if let Some((first, rest)) = args.split_first() {
        let command: Gc<Command> = first.clone().try_into_rust_type()?;
        let function = command.function.clone();
        let _ = function.call(rest).await?;
    }
    Ok(Vec::new())
}

#[bridge(name = "command-name", lib = "(koru-command)")]
pub fn command_name(command: &Value) -> Result<Vec<Value>, Condition> {
    let command: Gc<Command> = command.clone().try_into_rust_type()?;

    Ok(vec![Value::from(command.name.clone())])
}

#[bridge(name = "command-description", lib = "(koru-command)")]
pub fn command_description(command: &Value) -> Result<Vec<Value>, Condition> {
    let command: Gc<Command> = command.clone().try_into_rust_type()?;

    Ok(vec![Value::from(command.description.clone())])
}

#[bridge(name = "command-create", lib = "(koru-command)")]
pub fn command_create(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((first, rest)) = args.split_first() else {
        return Err(Condition::type_error("String", "invalid"));
    };
    let name: String = first.clone().try_into()?;
    let Some((first, rest)) = rest.split_first() else {
        return Err(Condition::type_error("String", "invalid"));
    };
    let description: String = first.clone().try_into()?;
    let Some((first, rest)) = rest.split_first() else {
        return Err(Condition::type_error("Procedure", "invalid"));
    };
    let function: Procedure = first.clone().try_into()?;

    let mut arguments: Vec<ArgumentDef> = Vec::new();
    for arg in rest {
        let arg: String = arg.clone().try_into()?;
        match ArgumentDef::try_from(arg.as_str()) {
            Ok(x) => arguments.push(x),
            Err(msg) => {
                return Err(Condition::error(msg))
            }
        }
    }

    let command = Command::new(name, function, description, arguments);
    Ok(vec![Value::from(Record::from_rust_type(command))])
}
