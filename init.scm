(import (rnrs))
(import (koru-session))
(import (koru-command))
(import (major-mode))
(import (scheme text-edit-mode))

(define cursor-up
  (command-create
    "cursor-up"
    "Moves the primary cursor up"
    (lambda (keys) (command-apply text-edit-mode-cursor-up 0))
    "key-sequence"))

(define cursor-down
  (command-create
    "cursor-down"
    "Moves the primary cursor down"
    (lambda (keys) (command-apply text-edit-mode-cursor-down 0))
    "key-sequence"))

(define cursor-left
  (command-create
    "cursor-left"
    "Moves the primary cursor left"
    (lambda (keys) (command-apply text-edit-mode-cursor-left 0 #f))
    "key-sequence"))

(define cursor-right
  (command-create
    "cursor-right"
    "Moves the primary cursor right"
    (lambda (keys) (command-apply text-edit-mode-cursor-right 0 #f))
    "key-sequence"))

(define place-point-mark
  (command-create
    "place-point-mark"
    "Places a point mark at the primary cursor"
    (lambda (keys) (command-apply text-edit-mode-place-point-mark 0))
    "key-sequence"))

(define remove-mark
  (command-create
    "remove-mark"
    "Removes the mark at the primary cursor"
    (lambda (keys) (command-apply text-edit-mode-remove-mark 0))
    "key-sequence"))

(define insert-text
  (command-create
    "insert-text"
    "Inserts text at the primary cursor"
    (lambda (keys) (command-apply text-edit-mode-insert-at-cursor 0 "t"))
    "key-sequence"))

(define delete-back
  (command-create
    "delete-back"
    "Deletes text before the primary cursor"
    (lambda (keys) (command-apply text-edit-mode-delete-before-cursor 0))
    "key-sequence"))

(add-hook "file-open" "text-edit-mode" text-edit-mode-file-open-hook)


(add-key-mapping "UP" cursor-up)
(add-key-mapping "DOWN" cursor-down)
(add-key-mapping "LEFT" cursor-left)
(add-key-mapping "RIGHT" cursor-right)
(add-key-mapping "m" place-point-mark)
(add-key-mapping "r" remove-mark)
(add-key-mapping "t" insert-text)
(add-key-mapping "BS" delete-back)