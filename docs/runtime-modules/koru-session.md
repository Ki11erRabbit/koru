# koru-session
These functions are for manipulating the state of the editor directly.

## `create-hook`
Creates a new hook that can be subscribed to when activated.

### Inputs
- hook-name: String, the name of the new hook to be created.

### Outputs
None

### Behavior
This will register a new hook with the runtime.
If the hook already exists, it will destroy the previous callbacks.

### Example
```scheme
(create-hook "my-hook")
```

## `destroy-hook`
Removes a hook from the runtime, removing callbacks.

### Inputs
- hook-name: String, the name of the hook to be destroyed.

### Outputs
None

### Behavior
This will remove the hook from the editor's state.
It will also remove all subscribers to the hook.

### Example
```scheme
(destroy-hook "my-hook")
```

## `add-hook`
Adds a new listener to a specified hook.

### Inputs
- hook-name: String, the name of the hook to respond to.
- callback-name: String, the name of the callback (used for deregistration purposes).
- callback: Procedure, a function that takes in the arguments of a hook and will be called when the hook activates.

### Outputs
None

### Behavior
This will register a callback to a hook.
If the `callback-name` overlaps with another callback, then the old callback will be replaced.

### Example
```scheme
(add-hook "my-hook" "my-callback" (lambda () (something)))
```

## `remove-hook`
Unregisters a callback from a given hook.

### Inputs
- hook-name: String, the name of the particular hook.
- callback-name: String, the name of the callback to be removed.

### Outputs
None

### Behavior
This will remove a callback from a given hook.
If the hook or callback does not exist, then this function does nothing.

### Example
```scheme
(remove-hook "my-hook" "my-callback")
```

## `emit-hook`
Triggers all callbacks with a particular hook.

### Inputs
- hook-name: String, the name of the hook to trigger.
- rest: Argument List, the arguments to pass into the hook.

### Outputs
None

### Behavior
This will trigger all callbacks associated with the hook.
Order is not guaranteed.

### Example
```scheme
(emit-hook "my-hook" 1 2 3 4)
```


## `add-key-binding`
Adds a global keybinding to the editor

### Inputs
- key-string: String, sequence of keys separated by spaces.
- command: Command, the command to execute on the key input.

### Outputs
None

### Behavior
This will load a new keybinding into the global editor state.
If the keybinding already existed, this will overwrite it.

### Example
```scheme
(add-key-binding "C-x s" save-command)
```

## `add-special-key-binding`
Adds a special keybinding to the editor

### Inputs
- key-string: String, a single key to execute the command on.
- command: Command, the command to execute on the key input.

### Outputs
None

### Behavior
This will load a new keybinding into the special editor.
Special keybindings only take one key and are always matched first.
This bypasses key sequencing.

### Example
```scheme
(add-special-key-binding "C-g" cancel-command)
```

## `add-key-map`
Adds a new keymap to the editor.
This should be used generally to add new keybindings.

### Inputs
- keymap-name: String, an identifier for the keymap.
- key-map: KeyMap, holds the key bindings to commands.

### Outputs
None

### Behavior
This adds a new keymap to the editor.
This will overwrite any previous keymaps if there are any.

### Example
```scheme
(add-key-map "my-keymap" my-keymap)
```

## `remove-key-map`
Removes a new keymap to the editor.

### Inputs
- keymap-name: String, an identifier for the keymap.

### Outputs
None

### Behavior
This adds a new keymap to the editor.
This will overwrite any previous keymaps if there are any.

### Example
```scheme
(add-key-map "my-keymap" my-keymap)
```

## `flush-key-buffer`
Clears the key buffer.

### Inputs
None

### Outputs
None

### Behavior
This empties the key buffer and does nothing if the buffer is empty.

### Example
```scheme
(add-key-map "my-keymap" my-keymap)
```