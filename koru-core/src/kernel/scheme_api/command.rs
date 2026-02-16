use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use scheme_rs::exceptions::Exception;
use scheme_rs::gc::{Gc, Trace};
use scheme_rs::proc::Procedure;
use scheme_rs::records::{rtd, Record, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::symbols::Symbol;
use scheme_rs::value::Value;
use tokio::sync::RwLock;

struct CommandTreeNode {
    command: Option<Gc<Command>>,
    children: HashMap<char, CommandTreeNode>,
}

impl CommandTreeNode {
    pub fn new() -> Self {
        Self {
            command: None,
            children: HashMap::new(),
        }
    }

    pub fn lookup(&self, c: &char) -> Option<&CommandTreeNode> {
        self.children.get(c)
    }

    pub fn lookup_mut(&mut self, c: &char) -> Option<&mut CommandTreeNode> {
        self.children.get_mut(c)
    }

    pub fn insert(&mut self, c: char) {
        self.children.insert(c, CommandTreeNode::new());
    }

    pub fn contains(&self, c: &char) -> bool {
        self.children.contains_key(c)
    }

    pub fn contains_command(&self) -> bool {
        self.command.is_some()
    }

    pub fn set_command(&mut self, command: Gc<Command>) {
        self.command = Some(command);
    }

    pub fn get_command(&self) -> Option<Gc<Command>> {
        self.command.clone()
    }
}

static COMMAND_TREE: LazyLock<Arc<RwLock<CommandTree>>> = LazyLock::new(|| {
    Arc::new(RwLock::new(CommandTree::new()))
});

/// A prefix tree (trie) that stores all registered commands.
pub struct CommandTree {
    root: CommandTreeNode,
}

impl CommandTree {
    fn new() -> Self {
        Self {
            root: CommandTreeNode::new(),
        }
    }

    fn insert_internal(&mut self, command_name: &str, command: Gc<Command>) {
        let mut node = &mut self.root;
        for c in command_name.chars() {
            if !node.contains(&c) {
                node.insert(c);
            }
            let Some(child) = node.lookup_mut(&c) else {
                unreachable!("We should have just inserted character '{c}'");
            };
            child.insert(c);
            node = child;
        }
        node.set_command(command);
    }

    fn get_internal(&self, command_name: &str) -> Option<Gc<Command>> {
        let mut node = &self.root;
        for c in command_name.chars() {
            let Some(child) = node.lookup(&c) else {
                return None;
            };
            node = child;
        }
        node.get_command()
    }

    /// Inserts a new command into the global tree.
    pub async fn insert(command_name: &str, command: Gc<Command>) {
        let mut guard = COMMAND_TREE.write().await;
        guard.insert_internal(command_name, command);
    }

    /// Looks up a command from the global tree.
    pub async fn lookup(command_name: &str) -> Option<Gc<Command>> {
        let guard = COMMAND_TREE.read().await;
        guard.get_internal(command_name)
    }
}


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
        rtd!(name: "&ArgumentDef", sealed: true)
    }
}



#[derive(Clone, Debug, Trace)]
pub struct Command {
    name: Symbol,
    function: Procedure,
    description: String,
    arguments: Vec<ArgumentDef>,
}

impl Command {

    pub fn new(
        name: Symbol,
        function: Procedure,
        description: String,
        arguments: Vec<ArgumentDef>
    ) -> Self {
        Command {
            name,
            function,
            description,
            arguments,
        }
    }
    pub fn name(&self) -> Arc<str> {
        self.name.to_str()
    }
    
    pub fn command(&self) -> &Procedure {
        &self.function
    }

    /// Executes the command for an arg list of strings.
    /// This parses the args according to the argument definition.
    pub async fn execute(&self, args: &[String]) -> Result<Vec<Value>, Exception> {
        let mut function_args = Vec::with_capacity(args.len());
        let mut args = args.iter();

        for arg_def in &self.arguments {
            match arg_def {
                ArgumentDef::Text => {
                    let Some(arg) = args.next() else {
                        return Err(Exception::error("Not enough arguments for command"))
                    };
                    function_args.push(Value::from(arg.to_string()));
                }
                ArgumentDef::Number => {
                    let Some(arg) = args.next() else {
                        return Err(Exception::error("Not enough arguments for command"))
                    };
                    if let Ok(number) = arg.parse::<usize>() {
                        function_args.push(Value::from(number));
                    } else if let Ok(number) = arg.parse::<f64>() {
                        function_args.push(Value::from(number));
                    } else {
                        return Err(Exception::error("Value not convertible to number"))
                    }
                }
                ArgumentDef::Path => {
                    let Some(arg) = args.next() else {
                        return Err(Exception::error("Not enough arguments for command"))
                    };
                    function_args.push(Value::from(arg.to_string()));
                }
                ArgumentDef::KeyPress => {
                    let Some(arg) = args.next() else {
                        return Err(Exception::error("Not enough arguments for command"))
                    };
                    function_args.push(Value::from(arg.to_string()));
                }
                ArgumentDef::KeySequence => {
                    let Some(arg) = args.next() else {
                        return Err(Exception::error("Not enough arguments for command"))
                    };
                    function_args.push(Value::from(arg.to_string()));
                }
                ArgumentDef::Boolean => {
                    let Some(arg) = args.next() else {
                        return Err(Exception::error("Not enough arguments for command"))
                    };
                    if let Ok(boolean) = arg.parse::<bool>() {
                        function_args.push(Value::from(boolean));
                    } else {
                        return Err(Exception::error("Value not convertible to boolean"))
                    }
                }
                ArgumentDef::Variable(x) => {
                    while let Some(arg) = args.next() {
                        match x.as_ref() {
                            ArgumentDef::Text => {
                                function_args.push(Value::from(arg.to_string()));
                            }
                            ArgumentDef::Number => {
                                if let Ok(number) = arg.parse::<usize>() {
                                    function_args.push(Value::from(number));
                                } else if let Ok(number) = arg.parse::<f64>() {
                                    function_args.push(Value::from(number));
                                } else {
                                    return Err(Exception::error("Value not convertible to number"))
                                }
                            }
                            ArgumentDef::Path => {
                                function_args.push(Value::from(arg.to_string()));
                            }
                            ArgumentDef::KeyPress => {
                                function_args.push(Value::from(arg.to_string()));
                            }
                            ArgumentDef::KeySequence => {
                                function_args.push(Value::from(arg.to_string()));
                            }
                            ArgumentDef::Boolean => {
                                if let Ok(boolean) = arg.parse::<bool>() {
                                    function_args.push(Value::from(boolean));
                                } else {
                                    return Err(Exception::error("Value not convertible to boolean"))
                                }
                            }
                            ArgumentDef::Variable(_) => {
                                unreachable!("It should not be possible to have a variable variable arg def")
                            }
                        }
                    }
                }
            }
        }
        self.function.call(&function_args).await
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
        rtd!(name: "&Command", sealed: true)
    }
}

#[bridge(name = "command=", lib = "(koru-command)")]
pub fn equal_command(args: &[Value]) -> Result<Vec<Value>, Exception> {
    if let Some((first, rest)) = args.split_first() {
        let first: Gc<Command> = first.clone().try_to_rust_type()?;
        for next in rest {
            let next: Gc<Command> = next.clone().try_to_rust_type()?;
            if first != next {
                return Ok(vec![Value::from(false)]);
            }
        }
    }
    Ok(vec![Value::from(true)])
}

#[bridge(name = "command-apply", lib = "(koru-command)")]
pub async fn command_apply(args: &[Value]) -> Result<Vec<Value>, Exception> {
    if let Some((first, rest)) = args.split_first() {
        let command: Gc<Command> = first.clone().try_to_rust_type()?;
        let function = command.function.clone();
        let args = function.call(rest).await?;
        return Ok(args);
    }
    Ok(Vec::new())
}

#[bridge(name = "command-name", lib = "(koru-command)")]
pub fn command_name(command: &Value) -> Result<Vec<Value>, Exception> {
    let command: Gc<Command> = command.clone().try_to_rust_type()?;

    Ok(vec![Value::from(command.name.clone())])
}

#[bridge(name = "command-description", lib = "(koru-command)")]
pub fn command_description(command: &Value) -> Result<Vec<Value>, Exception> {
    let command: Gc<Command> = command.clone().try_to_rust_type()?;

    Ok(vec![Value::from(command.description.clone())])
}

#[bridge(name = "command-create", lib = "(koru-command)")]
pub async fn command_create(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((name, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let Some((description, rest)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let Some((procedure, rest)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let name: Symbol = name.clone().try_into()?;
    let description: String = description.clone().try_into()?;
    let function: Procedure = procedure.clone().try_into()?;
    let mut hide = false;

    let mut arguments: Vec<ArgumentDef> = Vec::new();
    for arg in rest {
        if let Ok(hide_command) = TryInto::<bool>::try_into(arg.clone()) {
            hide = hide_command;
            continue;
        }

        let arg: Symbol = match arg.clone().try_into() {
            Ok(arg) => Ok(arg),
            Err(err) => {
                match arg.clone().try_into() {
                    Ok(arg) => {
                        let arg: bool = arg;
                        hide = arg;
                        continue;
                    }
                    _ => Err(err)
                }
            }
        }?;
        match ArgumentDef::try_from(arg.to_str().as_ref()) {
            Ok(x) => arguments.push(x),
            Err(msg) => {
                return Err(Exception::error(msg))
            }
        }
    }

    let command = Command::new(name, function, description, arguments);
    let command = Value::from(Record::from_rust_type(command));

    if !hide {
        let name_str = name.to_str();
        let command: Gc<Command> = command.clone().try_to_rust_type()?;
        CommandTree::insert(name_str.as_ref(), command).await;
    }

    Ok(vec![command])
}
