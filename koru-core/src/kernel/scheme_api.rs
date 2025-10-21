use std::mem::ManuallyDrop;
use std::sync::LazyLock;
use guile_rs::{Smob, SmobData, SmobDrop, SmobEqual, SmobPrint, SmobSize};
use guile_rs::scheme_object::{SchemeObject, SchemeProcedure};

#[derive(Clone)]
pub enum ArgumentDef {
    Text,
    Number,
    Path,
    Variable(Box<ArgumentDef>),
}

impl From<ArgumentDef> for &str {
    fn from(def: ArgumentDef) -> Self {
        match def {
            ArgumentDef::Text => "text",
            ArgumentDef::Number => "number",
            ArgumentDef::Path => "path",
            ArgumentDef::Variable(x) => {
                match x.as_ref() {
                    ArgumentDef::Text => "variable:text",
                    ArgumentDef::Number => "variable:number",
                    ArgumentDef::Path => "variable:path",
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
            ArgumentDef::Variable(x) => {
                match x.as_ref() {
                    ArgumentDef::Text => "variable:text",
                    ArgumentDef::Number => "variable:number",
                    ArgumentDef::Path => "variable:path",
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
            "variable:text" => Ok(ArgumentDef::Variable(Box::new(ArgumentDef::Text))),
            "variable:number" => Ok(ArgumentDef::Variable(Box::new(ArgumentDef::Number))),
            "variable:path" => Ok(ArgumentDef::Variable(Box::new(ArgumentDef::Path))),
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

#[derive(Clone)]
pub struct Command {
    internal: ManuallyDrop<CommandInternal>,
}

#[derive(Clone)]
struct CommandInternal {
    name: String,
    function: SchemeProcedure,
    description: String,
    arguments: Vec<ArgumentDef>
}

pub static COMMAND_SMOB: LazyLock<Smob<Command>> = LazyLock::new(||{
    Smob::register("Command")
});

impl SmobPrint for Command {
    fn print(&self) -> String {
        let mut output = format!("#<Command {} {} ", (*self.internal).name, (*self.internal).description);

        for (i, arg) in (*self.internal).arguments.iter().enumerate() {
            match arg {
                ArgumentDef::Text => {
                    output.push_str("text");
                }
                ArgumentDef::Number => {
                    output.push_str("number");
                }
                ArgumentDef::Path => {
                    output.push_str("path");
                }
                ArgumentDef::Variable(x) => {
                    let out = match x.as_ref() {
                        ArgumentDef::Text => "variable:text",
                        ArgumentDef::Number => "variable:number",
                        ArgumentDef::Path => "variable:path",
                        _ => unreachable!("invalid variable arg")
                    };
                    output.push_str(out);
                }
            }
            if i + 1 < (*self.internal).arguments.len() {
                output.push_str(" ");
            }
        }
        output.push_str(">");
        output
    }
}

impl SmobDrop for Command {
    fn drop(&mut self) -> usize {
        let name_capacity = (self.internal).name.capacity();
        let description_capacity = (self.internal).description.capacity();
        let argument_capacity = (self.internal).arguments.capacity();

        unsafe {
            ManuallyDrop::drop(&mut self.internal);
        }
        name_capacity + argument_capacity + description_capacity
    }

    fn heap_size(&self) -> usize {
        let name_capacity = (self.internal).name.capacity();
        let description_capacity = (self.internal).description.capacity();
        let argument_capacity = (self.internal).arguments.capacity();

        name_capacity + argument_capacity + description_capacity
    }
}

impl SmobSize for Command {}

impl SmobEqual for Command {
    fn eq(&self, other: SchemeObject) -> bool {
        
        let Some(other) = other.cast_smob(COMMAND_SMOB.clone()) else {
            return false;
        };

        (self.internal).name == (other.internal).name
    }
}

impl SmobData for Command {}