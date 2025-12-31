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

