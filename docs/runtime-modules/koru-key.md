# koru-key

## Types

### KeyMap

#### Constructors
##### `key-map-create`
Creates an empty keymap.
Can take in an optional parameter for a default action.
###### Inputs
- default: Optional Command, the command to execute when there are no key matches.
###### Outputs
KeyMap: The resulting keymap
###### Errors
None
###### Behavior
Simple constructor
###### Example
```scheme
; no default so sparse
(key-map-create)
; default so dense
(key-map-create my-command)
```

#### Accessors
None

#### Methods
##### `key-map-insert`
Adds a new keybinding to the keymap.
###### Inputs
- keymap: KeyMap, the keymap to update.
- key-sequence: String, a sequence of key presses to bind to.
- command: Command, the command to execute from the key sequence.
###### Outputs
None
###### Errors
None
###### Behavior
Mutates the keymap

###### Example
```scheme
(define keymap (key-map-create))

(key-map-insert keymap "C-x v" my-command)
```

##### `key-map-delete`
Removes a keybinding from the keymap.
###### Inputs
- keymap: KeyMap, the keymap to update.
- key-sequence: String, a sequence of key presses to remove.
###### Outputs
None
###### Errors
None
###### Behavior
Mutates the keymap

###### Example
```scheme
(define keymap (key-map-create))

(key-map-insert keymap "C-x v" my-command)
(key-map-delete keymap "C-x v")
```

### KeyPress

#### Constructors
##### `string->keypress`
Creates a new KeyPress from a string
###### Inputs
- string: String, the string to be converted into a keypress
###### Outputs
KeyPress: the resulting keypress from the string.
###### Errors
Raises an error if the string couldn't be converted into a keypress.
###### Behavior
Simple constructor that can fail.
###### Example
```scheme
(string->keypress "ENTER")
```

#### Accessors
None

#### Methods
None

### Functions

##### `string->key-sequence`
Converts a string into a list of keypresses.

###### Inputs
- string: String, the string holding a sequence of keypresses
###### Outputs
List KeyPress: A list of keypresses from the input string.
###### Errors
Raises error if one of the keypress strings is invalid.
###### Behavior
Simple constructor that can fail.
###### Example
```scheme
(string->key-sequence "C-x u")
```