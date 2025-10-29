use std::mem::ManuallyDrop;
use std::sync::LazyLock;
use guile_rs::{guile_misc_error, guile_wrong_type_arg, Guile, Module, SchemeValue, SmobData, SmobTag};
use guile_rs::scheme_object::{SchemeObject, SchemeProcedure, SchemeSmob, SchemeString};



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
    name: String,
    function: SchemeProcedure,
    description: String,
    arguments: Vec<ArgumentDef>,
}

impl Command {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}


pub static COMMAND_SMOB_TAG: LazyLock<SmobTag<Command>> = LazyLock::new(||{
    SmobTag::register("Command")
});

impl SmobData for Command {
    fn print(&self) -> String {
        let mut output = format!("#<Command {} ", self.name);

        for (i, arg) in self.arguments.iter().enumerate() {
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
            if i + 1 < self.arguments.len() {
                output.push_str(" ");
            }
        }
        output.push_str(">");
        output
    }

    fn heap_size(&self) -> usize {
        let name_capacity = self.name.capacity();
        let description_capacity = self.description.capacity();
        let argument_capacity = self.arguments.capacity();

        name_capacity + argument_capacity + description_capacity
    }

    fn eq(&self, other: SchemeObject) -> bool {
        let Some(other) = other.cast_smob(COMMAND_SMOB_TAG.clone()) else {
            return false;
        };

        self.name == other.borrow().name
    }
}

extern "C" fn command_apply(command: SchemeValue, rest: SchemeValue) -> SchemeValue {
    let Some(command) = SchemeObject::from(command).cast_smob(COMMAND_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("command-apply", 1, command);
    };

    command.borrow().function.call1(rest).into()
}

extern "C" fn command_name(command: SchemeValue) -> SchemeValue {
    let Some(command) = SchemeObject::from(command).cast_smob(COMMAND_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("command-name", 1, command);
    };

    let out: SchemeObject = SchemeString::new(command.borrow().name.clone()).into();
    out.into()
}

extern "C" fn command_description(command: SchemeValue) -> SchemeValue {
    let Some(command) = SchemeObject::from(command).cast_smob(COMMAND_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("command-description", 1, command);
    };

    let out: SchemeObject = SchemeString::new(command.borrow().description.clone()).into();
    out.into()
}

extern "C" fn command_arguments_add(command: SchemeValue, rest: SchemeValue) -> SchemeValue {
    let Some(command) = SchemeObject::from(command).cast_smob(COMMAND_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("command-add-arguments", 1, command);
    };
    let Some(list) = SchemeObject::from(rest).cast_list() else {
        guile_wrong_type_arg!("command-add-arguments", 2, rest);
    };

    for arg in list.iter() {
        let Some(symbol) = arg.cast_symbol() else {
            guile_misc_error!("command-add-arguments", "expected a symbol");
        };
        let Ok(arg_def) = ArgumentDef::try_from(symbol.to_string()) else {
            guile_misc_error!(
                "command-add-arguments", 
                "expected one of 'text, 'number, 'path, 'variable:text, 'variable:number', or 'variable:path'"
            );
        };
        command.borrow_mut().arguments.push(arg_def);
    }

    SchemeValue::undefined()
}

extern "C" fn command_create(name: SchemeValue, description: SchemeValue, function: SchemeValue) -> SchemeValue {
    let Some(name) = SchemeObject::from(name).cast_string() else {
        guile_wrong_type_arg!("command-create", 1, name);
    };
    let Some(description) = SchemeObject::from(description).cast_string() else {
        guile_wrong_type_arg!("command-create", 2, description);
    };
    let Some(function) = SchemeObject::from(function).cast_procedure() else {
        guile_wrong_type_arg!("command-create", 3, function);
    };

    let name = name.to_string();
    let description = description.to_string();

    let command = Command {
        name,
        description,
        function,
        arguments: Vec::new(),
    };

    let smob = COMMAND_SMOB_TAG.make(command);

    <SchemeSmob<_> as Into<SchemeObject>>::into(smob).into()
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