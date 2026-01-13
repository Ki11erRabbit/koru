# koru-session
These functions are for manipulating the state of the editor directly.

## Functions

### `create-hook`
Creates a new hook that can be subscribed to when activated.

###### Inputs
- hook-name: Symbol, the name of the new hook to be created.

###### Outputs
None
###### Errors
None
###### Behavior
This will register a new hook with the runtime.
If the hook already exists, it will destroy the previous callbacks.

###### Example
```scheme
(create-hook 'my-hook)
```

### `destroy-hook`
Removes a hook from the runtime, removing callbacks.

###### Inputs
- hook-name: Symbol, the name of the hook to be destroyed.

###### Outputs
None
###### Errors

###### Behavior
This will remove the hook from the editor's state.
It will also remove all subscribers to the hook.

###### Example
```scheme
(destroy-hook 'my-hook)
```

### `add-hook`
Adds a new listener to a specified hook.

###### Inputs
- hook-name: Symbol, the name of the hook to respond to.
- callback-name: Symbol, the name of the callback (used for deregistration purposes).
- callback: Procedure, a function that takes in the arguments of a hook and will be called when the hook activates.

###### Outputs
None
###### Errors
None

###### Behavior
This will register a callback to a hook.
If the `callback-name` overlaps with another callback, then the old callback will be replaced.

###### Example
```scheme
(add-hook 'my-hook 'my-callback (lambda () (something)))
```

### `remove-hook`
Unregisters a callback from a given hook.

###### Inputs
- hook-name: Symbol, the name of the particular hook.
- callback-name: String, the name of the callback to be removed.

###### Outputs
None
###### Errors
None

###### Behavior
This will remove a callback from a given hook.
If the hook or callback does not exist, then this function does nothing.

###### Example
```scheme
(remove-hook 'my-hook 'my-callback)
```

### `emit-hook`
Triggers all callbacks with a particular hook.

###### Inputs
- hook-name: Symbol, the name of the hook to trigger.
- rest: Argument List, the arguments to pass into the hook.

###### Outputs
None
###### Errors
None

###### Behavior
This will trigger all callbacks associated with the hook.
Order is not guaranteed.

###### Example
```scheme
(emit-hook 'my-hook 1 2 3 4)
```


### `add-key-binding`
Adds a global keybinding to the editor

###### Inputs
- key-string: String, sequence of keys separated by spaces.
- command: Command, the command to execute on the key input.

###### Outputs
None
###### Errors
None

###### Behavior
This will load a new keybinding into the global editor state.
If the keybinding already existed, this will overwrite it.

###### Example
```scheme
(add-key-binding "C-x s" save-command)
```

### `remove-key-binding`
Removes a global keybinding to the editor

###### Inputs
- key-string: String, sequence of keys separated by spaces.

###### Outputs
None
###### Errors
None

###### Behavior
This will remove a key binding from the global state.

###### Example
```scheme
(remove-key-binding "C-x s")
```

### `add-special-key-binding`
Adds a special keybinding to the editor

###### Inputs
- key-string: String, a single key to execute the command on.
- command: Command, the command to execute on the key input.

###### Outputs
None
###### Errors
None

###### Behavior
This will load a new keybinding into the special editor.
Special keybindings only take one key and are always matched first.
This bypasses key sequencing.

###### Example
```scheme
(add-special-key-binding "C-g" cancel-command)
```

### `remove-special-key-binding`
Removes a special keybinding to the editor

###### Inputs
- key-string: String, a single key to execute the command on.

###### Outputs
None
###### Errors
None

###### Behavior
The keybinding is removed from the special group.
Special keybindings only take one key and are always matched first.

###### Example
```scheme
(remove-special-key-binding "C-g")
```

### `add-key-map`
Adds a new keymap to the editor.
This should be used generally to add new keybindings.

###### Inputs
- keymap-name: Symbol, an identifier for the keymap.
- key-map: KeyMap, holds the key bindings to commands.

###### Outputs
None
###### Errors
None

###### Behavior
This adds a new keymap to the editor.
This will overwrite any previous keymaps if there are any.

###### Example
```scheme
(add-key-map 'my-keymap my-keymap)
```

### `remove-key-map`
Removes a new keymap to the editor.

###### Inputs
- keymap-name: Symbol, an identifier for the keymap.

###### Outputs
None
###### Errors
None

###### Behavior
This adds a new keymap to the editor.
This will overwrite any previous keymaps if there are any.

###### Example
```scheme
(remove-key-map 'my-keymap)
```

### `flush-key-buffer`
Clears the key buffer.

###### Inputs
None

###### Outputs
None

###### Errors
None

###### Behavior
This empties the key buffer and does nothing if the buffer is empty.

###### Example
```scheme
(flush-key-buffer)
```

### `command-bar-left`
Moves the cursor in the command bar to the left.

###### Inputs
None

###### Outputs
None

###### Errors
None

###### Behavior
Moves the internal cursor in the command bar to the left.
If the command bar is already at the leftmost position, this function does nothing.

###### Example
```scheme
(command-bar-left)
```

### `command-bar-right`
Moves the cursor in the command bar to the right.

###### Inputs
None

###### Outputs
None

###### Errors
None

###### Behavior
Moves the internal cursor in the command bar to the right.
If the cursor is already at the rightmost position, then this does nothing.

###### Example
```scheme
(command-bar-right)
```

### `command-bar-delete-back`
Deletes one character before the cursor.

###### Inputs
None

###### Outputs
None

###### Errors
None

###### Behavior
This deletes the character before the cursor and moves the cursor one space to compensate for the change.

###### Example
```scheme
(command-bar-delete-back)
```

### `command-bar-delete-forward`
Deletes one character after the cursor.

###### Inputs
None

###### Outputs
None

###### Errors
None

###### Behavior
Deletes the character at the cursor and does not move the cursor at all.

###### Example
```scheme
(command-bar-delete-forward)
```

### `command-bar-take`
Takes the text out of the command bar.

###### Inputs
None

###### Outputs
String: The text that was in the command bar.

###### Errors
None

###### Behavior
This causes the command bar to lose its text.

###### Example
```scheme
(let ((text (command-bar-take)))
  (do-something text))
```

### `command-bar-get`
Gets a copy of the text in the command bar.

###### Inputs
None

###### Outputs
String: The text in the command bar.

###### Errors
None

###### Behavior
This simply clones the text from the command bar.

###### Example
```scheme
(let ((text (command-bar-get)))
  (do-something text))
```

### `command-bar-insert`
Inserts a string into the command bar.

###### Inputs
string: String, the string to insert.

###### Outputs
None

###### Errors
None

###### Behavior
This inserts the string at the cursor position in the text buffer and moves the cursor forward the number of characters in the input string.

###### Example
```scheme
(command-bar-insert "input-string")
```

### `command-bar-insert-key`
Inserts a string into the command bar via a key sequence.

###### Inputs
key-sequence: List KeyPress, a list of key presses.

###### Outputs
- `#f` if the key-sequence is longer than 1
- `#t` if the key-sequence is 1

###### Errors
None

###### Behavior
This only works if the length of the list is one.
If the length of the list is greater than one then nothing happens.

###### Example
```scheme
(command-bar-insert-key keys)
```

### `command-bar-show`
Indicates to all running editor sessions that the command bar should be displayed.

###### Inputs
None

###### Outputs
None

###### Errors
None

###### Behavior
This communicates with all sessions to tell their frontends to display the command bar.

###### Example
```scheme
(command-bar-show)
```

### `command-bar-hide`
Indicates to all running editor sessions that the command bar should be displayed.

###### Inputs
None

###### Outputs
None

###### Errors
None

###### Behavior
This communicates with all sessions to tell their frontends to display the command bar.

###### Example
```scheme
(command-bar-hide)
```

### `command-bar-update`
Notifies the sessions that the command-bar has changed and needs to be updated.
It can also take in strings for added context in the command bar.

###### Inputs
- prefix: Optional String, what to display in front of the command bar.
- suffix: Optional String, what to display after the command bar.

###### Outputs
None

###### Errors
None

###### Behavior
This communicates with all sessions that their frontends should update the command bar.

###### Example
```scheme
(command-bar-update "Enter a command: ")
```