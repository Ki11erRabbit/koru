# koru-buffer

This module contains the APIs to interact with buffers directly.


##### `buffer-change-focus`
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

##### `buffer-from-path`
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

###### `buffer-create`
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

###### `major-mode-set!`
Sets the major mode of a given buffer
This should be called after buffer creation.

###### Inputs
- buffer-name: String, the name of a buffer that exists.
- major-mode: MajorMode, the major mode to set in the buffer.

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


###### `current-major-mode`
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
