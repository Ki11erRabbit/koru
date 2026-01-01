# major-mode

## Types

### MajorMode
The type that represents the major mode of a buffer.
#### Constructors
##### `major-mode-create`
Creates a new major mode.
###### Inputs
- name: String, the name of the major mode.
- draw: Procedure, A function that takes in the major mode and draws the buffer.
- data: Optional Any, state for the major mode instance.
###### Outputs
MajorMode: The resulting major mode instance.
###### Errors
None
###### Behavior
Simple Constructor
###### Example
```scheme
(major-mode-create "my-major-mode" my-draw #f)
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
Simple Accessor

#### Methods
None
