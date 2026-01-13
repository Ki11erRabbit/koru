# koru-buffer

This module contains the APIs to interact with buffers directly.

## Functions

### `buffer-change-focus`
Changes the currently focused buffer.

###### Inputs
- buffer-name: String

###### Outputs
None
###### Errors
None

###### Behavior
The buffer in focus will be changed to the one that matches the string.

###### Example
```scheme
(buffer-change-focus "my buffer")
```

### `buffer-from-path`
Creates a new buffer from a file, given its path.

###### Inputs
- path: String, an absolute or relative file path

###### Outputs
String: the name of the buffer created
###### Errors
An error message is raised if there is an error opening the file for reading.

###### Behavior
The path given will be canonicalized into an absolute path.
If the path does not match a file, then an error is reported.
If the path does match a file, then the buffer will be created with the contents of that file.
The name returned will be the absolute path of the file.

###### Example
```scheme
(let ((new-buffer (buffer-from-path "my-file.md")))
  (buffer-change-focus new-buffer))
```

### `buffer-create`
Creates a new buffer with a given name and an optional given contents.

###### Inputs
- name: String, The name of the buffer
- contents: Optional String, The contents of the new buffer.

###### Outputs
String: The name of the new buffer.
###### Errors
An error message is raised if there is an issue with creating the buffer.

###### Behavior
This function does not call out to the filesystem in any way.
It is a safe way to create a buffer programmatically.

###### Example
```scheme
(let ((new-buffer (buffer-create "my-buffer.md")))
  (buffer-change-focus new-buffer))
```

### `major-mode-set!`
Sets the major mode of a given buffer
This should be called after buffer creation.

###### Inputs
- buffer-name: String, the name of a buffer that exists.
- major-mode: MajorMode, the major mode to be set in the buffer.

###### Outputs
None
###### Errors
An error message if the buffer is not found.

###### Behavior
This will change the major mode of a buffer to the one specified.

###### Example
```scheme
(major-mode-set! "my-buffer" my-major-mode)
```


### `current-major-mode`
Fetches the major mode of the currently focused buffer.

###### Inputs
None

###### Outputs
MajorMode: The major mode associated with the buffer.
###### Errors
An error indicating that there is no current buffer.

###### Behavior
This fetches the current buffer's major mode and returns it.

###### Example
```scheme
(current-major-mode)
```

### `minor-mode-add`
Adds a minor mode to a particular buffer

###### Inputs
- buffer-name: String, the name of a buffer that exists.
- minor-mode: MinorMode, the minor mode to be added to the buffer.

###### Outputs
None
###### Errors
An error message if the buffer is not found.

###### Behavior
This will add a new minor mode to a buffer if it is found.

###### Example
```scheme
(minor-mode-add "my-buffer" "vi-mode" vi-mode)
```

### `minor-mode-get`
Gets a minor mode from the current buffer if it exists.

###### Inputs
- mode-name: Symbol, the name of a minor mode in the current buffer that exists.

###### Outputs
None
###### Errors
An error message if there is no current buffer or the minor mode is not found.

###### Behavior
This searches the current buffer for the minor mode.

###### Example
```scheme
(minor-mode-get 'vi-mode)
```

### `current-buffer-name`
Gets the name of the currently focused buffer.

###### Inputs
None

###### Outputs
- String: if there is a currently focused buffer.
- Null: if there isn't a currently focused buffer.
###### Errors
None
###### Behavior
This simply fetches the currently focused buffer

###### Example
```scheme
(current-buffer-name)
```

### `buffer-save`
Saves the current buffer if it is bound to a path.

###### Inputs
- buffer-name: String, the name of the buffer to save.

###### Outputs
None
###### Errors
- Buffer not found: if the buffer-name does not exist.
- IO Error: when there was an io error
- Buffer has no associated path: When there is no path associated with the buffer.
###### Behavior
This will overwrite the file on disk with the current contents of the buffer.

###### Example
```scheme
(buffer-save "my-buffer.txt")
```

### `buffer-save-as`
Saves the current buffer and binds it to a path.

###### Inputs
- buffer-name: String, the name of the buffer to save.
- path: String, the path to bind the buffer to.

###### Outputs
None
###### Errors
- Buffer not found: if the buffer-name does not exist.
- IO Error: when there was an io error
###### Behavior
This should always succeed unless the path to the new file name does not exist.

###### Example
```scheme
(buffer-save-as "my-buffer.txt" "my-buffer.md")
```

### `buffer-get-path`
Fetches the path from a buffer if the buffer exists or if the path doesn't exist.

###### Inputs
- buffer-name: String, the name of the buffer to get its path.

###### Outputs
String: If there is a path associated with the buffer.
Null: If there isn't a path associated with the buffer.
###### Errors
Buffer not found: if the buffer-name does not exist.
###### Behavior
This returns null if there is no file path associated with the buffer.

###### Example
```scheme
(buffer-get-path "my-buffer.txt")
```


### `is-current-buffer-set?`
Checks if the current buffer is set or not.

###### Inputs
None

###### Outputs
Boolean: `#t` if the current buffer is set, `#f` if the current buffer isn't set.
###### Errors
None
###### Behavior
This checks for the existence of a focused buffer.
The only time this should return false is at the very beginning of initialization.

###### Example
```scheme
(if (is-current-buffer-set?)
  (display "we have a current buffer!\n")
  (display "we don't have a current buffer yet!\n"))
```

### `plain-draw`
Draws a buffer by name with cursors

###### Inputs
- buffer-name: String, the name of the buffer to draw.
- cursors: Cursors, the cursors to draw with
###### Outputs
StyledFile: The styled file that represents the text.
###### Errors
An error is raised if the buffer can't be found.

###### Behavior
This accesses the buffer and causes it to render itself before being rendered.

###### Example
```scheme
(plain-draw "Temp" (cursors-create))
```