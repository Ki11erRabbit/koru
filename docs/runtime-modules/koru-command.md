# koru-command


## Types

### Command
Represents a command that can be bound to a keypress.
Generally, these shouldn't return anything.
However, they should return `#t` when the keybuffer should be flushed and `#f` if the keybuffer shouldn't be flushed and that another command should be executed instead.
By default, if there is nothing returned then it is assumed that the keybuffer should be flushed.

#### Constructors
##### `command-create`
Creates a new command.
Arguments are used to indicate to the runtime what should be completed when executing the command in the command bar.

Arguments can be one of the following:
- `"text"`
- `"number"`
- `"path"`
- `"key-press"`
- `"key-sequence"`
- `"boolean"`
- `"variable:text"`
- `"variable:number"`
- `"variable:path"`
- `"variable:key-press"`
- `"variable:key-sequence"`
- `"variable:boolean"`

###### Inputs
- name: String, the name of the command
- description: String, A description of the command
- function: Procedure, a function that takes the arguments for the command
- hide: Optional bool, Whether to let the command be completed or not. `#t` for to hide it. `#f` to not hide it (default).
- arguments: Rest String, a list of strings that are the type of arguments for the function. See above for possible values

###### Output
Command: The newly created command

###### Errors
If the arguments are not one of the possible values, then an error is raised.

###### Behavior
Currently just constructs the command, but in the future may register the command in the editor.

###### Example
```scheme
(command-create "delete" (lambda (keys) (delete-at-cursor)) "key-sequence")
(command-create "move" (lambda (keys) (delete-at-cursor)) #t "key-sequence")
```

#### Accessors

##### `command-name`
Fetches the name of the command.

###### Inputs
- command: Command, the command to get the name of.

###### Outputs
String: Command name
###### Errors
None
###### Behavior
Simple Accessor
###### Example
```scheme
(command-name my-command)
```

##### `command-description`
Fetches the description of the command.
###### Inputs
- command: Command, the command to get the description of.
###### Outputs
String: Command description.
###### Errors
None
###### Behavior
Simple Accessor
###### Example
```scheme
(command-description my-command)
```

#### Methods

##### `command=`
Equality between two commands
###### Inputs
- x: Command
- rest: Rest Command, a sequence of commands to compare against.
###### Outputs
Boolean: A boolean indicating that the rest of the commands are the same as x.
###### Errors
TypeError: if any of rest is not a command
###### Behavior
Short circuits on the first non-equal command.
###### Example
```scheme
(command= my-command my-other-command command) 
```

##### `command-apply`
Applies arguments to the specified command
###### Inputs
- command: Command, the command to execute the procedure
###### Outputs
- Any: Any type of the command.
- Boolean: `#t` if keybuffer should be flushed, `#f` if keybuffer shouldn't be flushed and another command should be executed instead.
- None

###### Errors
Any error the command may have.
###### Behavior
Executes the internal function.
###### Example
```scheme
(command-apply my-command "arg1" 2)
```
