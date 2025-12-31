# Runtime Modules

You can find the modules for interacting directly with the runtime yourself.

## Types

### Command
Represents a command that can be bound to a keypress.
Generally, these shouldn't return anything.
However, they should return `#t` when the keybuffer should be flushed and `#f` if the keybuffer shouldn't be flushed and that another command should be executed instead.
By default, if there is nothing returned then it is assumed that the keybuffer should be flushed.

#### Constructors
##### `command-create`
Creates a new command.
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
- arguments: Rest String, a list of strings that are the type of arguments for the function. See above for possible values

###### Output
Command: The newly created command

###### Errors
If the arguments are not one of the possible values, then an error is raised.

###### Behavior
Currently just constructs the command, but in the future may register the command in the editor.

#### Accessors

#### Methods

### KeyMap


### KeyPress



### MajorMode