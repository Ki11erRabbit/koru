(library (scheme text-edit-mode)
  (export text-edit-mode-cursor-up
    text-edit-mode-cursor-down
    text-edit-mode-cursor-left
    text-edit-mode-cursor-right
    text-edit-mode-place-point-mark
    text-edit-mode-remove-mark
    text-edit-mode-cursor-create
    text-edit-mode-cursor-destroy
    text-edit-mode-cursor-position
    text-edit-mode-cursor-count
    text-edit-mode-file-open-hook)
  (import (rnrs)
    (major-mode)
    (koru-command)
    (koru-session)
    (styled-text)
    (text-edit))

  (create-hook "text-edit-mode")

  (define text-edit-mode-cursor-up
    (command-create
      "text-edit-mode-cursor-up"
      "Moves the cursor at the index up"
      (lambda (index) (text-edit-move-cursor-up (current-major-mode) index))
      "number"))

  (define text-edit-mode-cursor-down
    (command-create
      "text-edit-mode-cursor-down"
      "Moves the cursor at the index down"
      (lambda (index) (text-edit-move-cursor-down (current-major-mode) index))
      "number"))

  (define text-edit-mode-cursor-left
    (command-create
      "text-edit-mode-cursor-left"
      "Moves the cursor at the index left with the possiblity of wrapping at the start"
      (lambda (index wrap) (text-edit-move-cursor-left (current-major-mode) index wrap))
      "number"
      "boolean"))

  (define text-edit-mode-cursor-right
    (command-create
      "text-edit-mode-cursor-right"
      "Moves the cursor at the index right with the possiblity of wrapping at the end"
      (lambda (index wrap) (text-edit-move-cursor-right (current-major-mode) index wrap))
      "number"))

    (define text-edit-mode-place-point-mark
      (command-create
        "text-edit-mode-place-point-mark"
        "Places a point mark at the position of the cursor indicated by the index"
        (lambda (index) (text-edit-place-point-mark-at-cursor (current-major-mode) index))
        "number"))

    (define text-edit-mode-remove-mark
      (command-create
        "text-edit-mode-remove-mark"
        "Removes a mark of the cursor indicated by the index"
        (lambda (index) (text-edit-remove-mark-from-cursor (current-major-mode) index))
        "number"))

    (define text-edit-mode-cursor-create
      (command-create
        "text-edit-mode-cursor-create"
        "Creates a new cursor at the indicated row and column"
        (lambda (row column) (text-edit-cursor-create (current-major-mode) row column))
        "number"
        "number"))

  (define text-edit-mode-cursor-destroy
    (command-create
      "text-edit-mode-cursor-destroy"
      "Removes the cursor at the indicated index"
      (lambda (index) (text-edit-cursor-destroy (current-major-mode) index))
      "number"))

    (define (text-edit-mode-cursor-position index)
      (text-edit-cursor-position (current-major-mode) index))

    (define (text-edit-mode-cursor-count)
      (text-edit-cursor-count (current-major-mode)))

  (define text-edit-mode-cursor-main
    (command-create
      "text-edit-mode-cursor-main"
      "Makes the cursor at the indicated index the new main cursor"
      (lambda (index) (text-edit-cursor-change-main (current-major-mode) index))
      "number"))

  (define (text-edit-mode-create buffer-name)
    (major-mode-create
      "TextEdit"
      text-edit-draw
      (text-edit-data-create buffer-name)))

  (define (text-edit-mode-file-open-hook buffer-name file-ext)
    (major-mode-set! buffer-name (text-edit-mode-create buffer-name))
    (emit-hook "text-edit-mode")))

