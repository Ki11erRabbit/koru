# koru-task

This module contains the APIs to interact with async tasks.

## Functions

### `spawn-task`
Takes in a parameterless function and runs it in the background

###### Inputs
- func: Procedure, the body of the task

###### Outputs
Number: an integer representing the task id
###### Errors
None

###### Behavior
The function will be run in the async runtime and can be awaited or canceled

###### Example
```scheme
(spawn-task (lambda () (do-something)))
```

### `await-task`
Takes in a task id and awaits the task until completion.

###### Inputs
- id: Int, the task to await

###### Outputs
The output of the function passed into `spawn-task`.
###### Errors
None

###### Behavior
This will block the current async task to wait for this task.
It will throw an exception if the task threw one.

###### Example
```scheme
(let ((id (spawn-task (lambda () (do-something)))))
    (await-task id))
```

### `cancel-task`
Takes in a task id to cancel a task.

###### Inputs
- id: Int, the task id

###### Outputs
None
###### Errors
None

###### Behavior
The task will be shutdown.

###### Example
```scheme
(let ((id (spawn-task (lambda () (do-something)))))
    (cancel-task id))
```