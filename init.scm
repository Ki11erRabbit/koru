(import (rnrs))
(import (koru-session))
(import (koru-command))
(import (major-mode))
(import (scheme text-edit-mode))

(define cursor-up
  (command-create
    "cursor-up"
    "Moves the primary cursor up"
    (lambda (keys) (command-apply text-edit-cursor-up 0 keys))
    "number"
    "key-sequence"))

(define cursor-down
  (command-create
    "cursor-down"
    "Moves the primary cursor down"
    (lambda (keys) (command-apply text-edit-cursor-down 0 keys))
    "number"
    "key-sequence"))

(define cursor-left
  (command-create
    "cursor-left"
    "Moves the primary cursor left"
    (lambda (keys) (command-apply text-edit-cursor-left 0 #f keys))
    "number"
    "boolean"
    "key-sequence"))

(define cursor-right
  (command-create
    "cursor-right"
    "Moves the primary cursor right"
    (lambda (keys) (command-apply text-edit-cursor-right 0 #f keys))
    "number"
    "key-sequence"))

(add-hook "file-open" "text-edit-mode" text-edit-file-open-hook)

(debug-print)

(add-key-mapping "UP" cursor-up)
(add-key-mapping "DOWN" cursor-down)
(add-key-mapping "LEFT" cursor-left)
(add-key-mapping "RIGHT" cursor-right)