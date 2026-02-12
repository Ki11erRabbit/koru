# koru-theme

This module contains the APIs to interact theming.

## Functions

### `color-definition-hex-set!`
Sets a color type to a hex value.

###### Inputs
- color-type: String, the type of color to be set
- color-value: int, the hex color value to use

###### Outputs
None
###### Errors
Errors if color-type is not a well-formed color type string.

###### Behavior
Updates the global setting for that color type.

###### Example
```scheme
(color-definition-hex-set! "Cursor" 0x8a8a8a)
```

### `color-definition-ansi-set!`
Sets a color type to an ANSI color value.

###### Inputs
- color-type: String, the type of color to be set
- color-value: int, the ANSI color to use

###### Outputs
None
###### Errors
Errors if color-type is not a well-formed color type string.

###### Behavior
Updates the global setting for that color type.

###### Example
```scheme
(color-definition-ansi-set! "Cursor" 10)
```
