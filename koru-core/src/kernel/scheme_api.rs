mod major_mode;

use std::mem::ManuallyDrop;
use std::sync::LazyLock;
use guile_rs::{Guile, Module, SchemeValue, Smob, SmobData, SmobDrop, SmobEqual, SmobPrint, SmobSize};
use guile_rs::scheme_object::{SchemeObject, SchemeProcedure, SchemeString, SchemeSymbol};



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
struct CommandInternal {
    name: String,
    function: SchemeProcedure,
    description: String,
    arguments: Vec<ArgumentDef>
}

#[derive(Clone)]
pub struct Command {
    internal: ManuallyDrop<CommandInternal>,
}

impl Command {
    pub fn name(&self) -> &str {
        self.internal.name.as_str()
    }
}


pub static COMMAND_SMOB: LazyLock<Smob<Command>> = LazyLock::new(||{
    Smob::register("Command")
});

impl SmobPrint for Command {
    fn print(&self) -> String {
        let mut output = format!("#<Command {} ", (*self.internal).name);

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

        (self.internal).name == other.internal.name
    }
}

impl SmobData for Command {}


extern "C" fn command_apply(command: SchemeValue, rest: SchemeValue) -> SchemeValue {
    let Some(command) = SchemeObject::new(command).cast_smob(COMMAND_SMOB.clone()) else {
        return SchemeObject::undefined().into()
    };

    command.internal.function.call1(rest).into()
}

extern "C" fn command_name(command: SchemeValue) -> SchemeValue {
    let Some(command) = SchemeObject::new(command).cast_smob(COMMAND_SMOB.clone()) else {
        return SchemeObject::undefined().into()
    };

    let out: SchemeObject = SchemeString::new(&command.internal.name).into();
    out.into()
}

extern "C" fn command_description(command: SchemeValue) -> SchemeValue {
    let Some(command) = SchemeObject::new(command).cast_smob(COMMAND_SMOB.clone()) else {
        return SchemeObject::undefined().into()
    };

    let out: SchemeObject = SchemeString::new(&command.internal.description).into();
    out.into()
}

extern "C" fn command_arguments_add(command: SchemeValue, rest: SchemeValue) -> SchemeValue {
    let Some(mut command) = SchemeObject::new(command).cast_smob(COMMAND_SMOB.clone()) else {
        return SchemeObject::undefined().into()
    };
    let Some(list) = SchemeObject::new(rest).cast_list() else {
        return SchemeObject::undefined().into()
    };
    
    for arg in list.iter() {
        let Some(string) = arg.cast_string() else {
            return SchemeObject::undefined().into()
        }; 
        let Ok(arg_def) = ArgumentDef::try_from(string.to_string()) else {
            return SchemeObject::undefined().into()
        };
        command.internal.arguments.push(arg_def);
    }
    
    SchemeObject::undefined().into()
}

extern "C" fn command_create(name: SchemeValue, description: SchemeValue, function: SchemeValue) -> SchemeValue {
    let Some(name) = SchemeObject::new(name).cast_string() else {
        return SchemeObject::undefined().into()
    };
    let Some(description) = SchemeObject::new(description).cast_string() else {
        return SchemeObject::undefined().into()
    };
    let Some(function) = SchemeObject::new(function).cast_procedure() else {
        return SchemeObject::undefined().into()
    };
    
    let name = name.to_string();
    let description = description.to_string();
    
    let command = Command {
        internal: ManuallyDrop::new(CommandInternal {
            name,
            description,
            function,
            arguments: Vec::new(),
        })
    };
    
    let smob = COMMAND_SMOB.make(command);
    
    smob.into()
}


pub fn koru_command_module() {
    Guile::define_fn("command-create", 3, 0, false, 
        command_create as extern "C" fn(SchemeValue, SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("command-apply", 1, 0, true, 
        command_apply as extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("command-name", 1, 0, false, 
        command_name as extern "C" fn(SchemeValue) -> SchemeValue
    );
    Guile::define_fn("command-description", 1, 0, false, 
        command_description as extern "C" fn(SchemeValue) -> SchemeValue
    );
    Guile::define_fn("command-add-arguments", 1, 0, true, 
        command_arguments_add as extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue
    );
    
    let mut module = Module::new("koru-command", Box::new(|_: &mut ()| {}));
    module.add_export("command-create");
    module.add_export("command-apply");
    module.add_export("command-name");
    module.add_export("command-description");
    module.add_export("command-add-arguments");
    module.export();
    module.define(&mut ());
}