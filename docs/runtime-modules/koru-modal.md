# koru-modal

## Types

### Modal
Represents the state needed for a text editor state.

#### Constructors
##### `modal-create`
Creates a new modal
###### Inputs
- initial-state: Symbol, the starting state for the modal.
- hook-name: Symbol, the hook to emit when the state is changed.
- callback: Procedure, a procedure executed when the state changes. This is called before the hook is emitted. It should take in the old and new states in that order.
###### Outputs
Modal: The resulting modal
###### Errors
None
###### Behavior
Simple constructor
###### Example
```scheme
(let ((minor-mode (minor-mode-create 'vi-mode (lambda (minor) ...) (lambda (minor) ...))))
  (let ((data (modal-create 'Normal 'vi-mode-change (lambda (old new) (proccess-mode minor-mode old new)))))
    (minor-mode-data-set! minor-mode data)
    minor-mode))
```

#### Accessors

##### `modal-state`
Returns the state of the modal
###### Inputs
- modal: Modal, the modal to fetch the state from.
###### Outputs
Symbol: The current state of the modal
###### Errors
None
###### Behavior
Simple Getter

###### Example
```scheme
(modal-state modal)
```


##### `modal-prefix-set!`
Changes the prefix in the modal.
###### Inputs
- modal: Modal, the modal to set the prefix.
- prefix: String, the new prefix to be set.
###### Outputs
None
###### Errors
None
###### Behavior
Mutates the modal

###### Example
```scheme
(modal-prefix-set! modal "Enter a command: ")
```

##### `modal-prefix`
Returns the prefix in the modal.
###### Inputs
- modal: Modal, the modal to get the prefix.
###### Outputs
String: The prefix contained in the Modal
###### Errors
None
###### Behavior
Simple Getter

###### Example
```scheme
(modal-prefix modal)
```

##### `modal-suffix-set!`
Set the suffix in the modal.
###### Inputs
- modal: Modal, the modal to set the suffix.
- suffix: String, the new suffix to be set.
###### Outputs
None
###### Errors
None
###### Behavior
Mutates the modal

###### Example
```scheme
(modal-suffix-set! modal "")
```

##### `modal-suffix`
Returns the suffix in the modal.
###### Inputs
- modal: Modal, the modal to get the suffix.
###### Outputs
String: The suffix contained in the Modal
###### Errors
None
###### Behavior
Simple Getter

###### Example
```scheme
(modal-suffix modal)
```

##### `modal-callback-set!`
Sets the command callback in the modal.
###### Inputs
- modal: Modal, the modal to set the command callback.
- suffix: Procedure or Null, The procedure to be executed when `modal-callback-apply` is called. This may be null to indicate that there is no callback. The procedure doesn't take any arguments.
###### Outputs
None
###### Errors
None
###### Behavior
Mutates the modal

###### Example
```scheme
(modal-callback-set! modal (lambda () (do-something)))
```

#### Methods
##### `modal-state-set!`
Changes the state of the modal.

This will first call the function passed at construction time.
Then it will emit the hook via the name provided at construction time.

###### Inputs
- modal: Modal, the modal to use.
- state: Symbol, the new state to set the modal to
###### Outputs
None
###### Errors
Any errors the callback or hook callbacks may cause.
###### Behavior
Mutates the Modal.
The internal state is changed before the functions are called.

The function is called first then the hook is emitted.
This is to ensure that whoever is using the modal may do their work first before other listeners.

###### Example
```scheme
(modal-state-set! modal 'Insert)
```


### Functions
None