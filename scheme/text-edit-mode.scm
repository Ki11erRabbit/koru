(import (rnrs))
(import (major-mode))
(import (koru-command))
(import (koru-session))
(import (styled-text))
(import (text-edit))

(create-hook "text-edit")

(define cursor-up
  (command-create
    "cursor-up"
    "Moves the primary cursor up"
    (lambda (keys) (text-edit-move-cursor-up (current-major-mode) 0))))

(define cursor-down
  (command-create
    "cursor-down"
    "Moves the primary cursor down"
    (lambda (keys) (text-edit-move-cursor-down (current-major-mode) 0))))

(define cursor-left
  (command-create
    "cursor-left"
    "Moves the primary cursor left"
    (lambda (keys) (text-edit-move-cursor-left (current-major-mode) 0 #f))))

(define cursor-right
  (command-create
    "cursor-right"
    "Moves the primary cursor right"
    (lambda (keys) (text-edit-move-cursor-right (current-major-mode) 0 #f))))

(add-key-mapping "UP" cursor-up)
(add-key-mapping "DOWN" cursor-down)
(add-key-mapping "LEFT" cursor-left)
(add-key-mapping "RIGHT" cursor-right)


(define (text-edit-create buffer-name)
  (major-mode-create
    "TextEdit"
    text-edit-draw
    (text-edit-data-create buffer-name)))

(define (text-edit-file-open-hook buffer-name file-ext)
  (major-mode-set! buffer-name (text-edit-create buffer-name))
  (emit-hook "text-edit"))

(add-hook "file-open" "text-edit-mode" text-edit-file-open-hook)