# major-mode

## Types

### MajorMode
The type that represents the major mode of a buffer.
#### Constructors
##### `major-mode-create`
Creates a new major mode.
###### Inputs
- name: Symbol, the name of the major mode.
- draw: Procedure, A function that takes in the major mode and draws the buffer.
- get-main-cursor: Procedure, A function that takes in the major mode and fetches the primary cursor.
- gain-focus: Procedure, A function that takes in the major mode and is used to restore state to the major mode when the current buffer changes.
- lose-focus: Procedure, A function that takes in the major mode and is used to save state when the current buffer changes.
- data: Optional Any, state for the major mode instance.
###### Outputs
MajorMode: The resulting major mode instance.
###### Errors
None
###### Behavior
Simple Constructor
###### Example
```scheme
(major-mode-create 'my-major-mode my-draw #f)
```

#### Accessors
##### `major-mode-data`
Accesses the data field from the major mode.
###### Inputs
- mode: MajorMode
###### Outputs
Any: The state associated with the major mode.
###### Errors
None
###### Behavior
Simple Getter

##### `major-mode-data-set!`
Sets the data field from the major mode.
###### Inputs
- mode: MajorMode
- data: Any
###### Outputs
None
###### Errors
None
###### Behavior
Simple Setter

#### Methods
None
