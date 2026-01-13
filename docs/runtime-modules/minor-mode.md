# minor-mode

## Types

### MinorMode

#### Constructors
##### `minor-mode-create`
Creates a new minor mode.
###### Inputs
- name: Symbol, the name of the minor mode.
- gain-focus: Procedure, a function that takes the minor mode and is called when the buffer that has this minor mode gains focus.
- lose-focus: Procedure, a function that takes the minor mode and is called when the buffer that contains the minor more loses focus.
- data: Optional Any, an optional state parameter.
###### Outputs
MinorMode: the newly created minor mode.
###### Errors
None
###### Behavior
Simple constructor
###### Example
```scheme
(minor-mode-create 'vi-mode vi-gain-focus vi-lose-focus "Normal")
```

#### Accessors
##### `minor-mode-data`
Fetches the data for the state of the minor mode.
###### Inputs
- minor-mode: MinorMode, the minor mode.
###### Outputs
Any: The state associated with the minor mode
###### Errors
None
###### Behavior
Simple Getter
###### Example
```scheme
(minor-mode-data vi-mode)
```

##### `minor-mode-data-set!`
Sets the data for the state of the minor mode.
###### Inputs
minor-mode: MinorMode, the minor mode.
data: Any, the new value to update the state of the minor mode.
###### Outputs
None
###### Errors
None
###### Behavior
Simple Setter
###### Example
```scheme
(minor-mode-data-set! emacs-mode)
```

#### Methods

## Functions