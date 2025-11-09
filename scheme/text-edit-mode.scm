(library (scheme text-edit-mode)
  (export text-edit-cursor-up
    text-edit-cursor-down
    text-edit-cursor-left
    text-edit-cursor-right
    text-edit-file-open-hook)
  (import (rnrs)
    (major-mode)
    (koru-command)
    (koru-session)
    (styled-text)
    (text-edit))
    (create-hook "text-edit")

  (define text-edit-cursor-up
    (command-create
      "text-edit-cursor-up"
      "Moves the cursor at the index up"
      (lambda (index keys) (text-edit-move-cursor-up (current-major-mode) index))
      "number"
      "key-sequence"))

  (define text-edit-cursor-down
    (command-create
      "text-edit-cursor-down"
      "Moves the cursor at the index down"
      (lambda (index keys) (text-edit-move-cursor-down (current-major-mode) index))
      "number"
      "key-sequence"))

  (define text-edit-cursor-left
    (command-create
      "text-edit-cursor-left"
      "Moves the cursor at the index left with the possiblity of wrapping at the start"
      (lambda (index wrap keys) (text-edit-move-cursor-left (current-major-mode) index wrap))
      "number"
      "boolean"
      "key-sequence"))

  (define text-edit-cursor-right
    (command-create
      "text-edit-cursor-right"
      "Moves the cursor at the index right with the possiblity of wrapping at the end"
      (lambda (index wrap keys) (text-edit-move-cursor-right (current-major-mode) index wrap))
      "number"
      "key-sequence"))

  (define (text-edit-create buffer-name)
    (major-mode-create
      "TextEdit"
      text-edit-draw
      (text-edit-data-create buffer-name)))

  (define (text-edit-file-open-hook buffer-name file-ext)
    (major-mode-set! buffer-name (text-edit-create buffer-name))
    (emit-hook "text-edit")))

