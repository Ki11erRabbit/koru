(library (scheme text-edit-mode)
  (export text-edit-mode-cursor-up
    text-edit-mode-cursor-down
    text-edit-mode-cursor-left
    text-edit-mode-cursor-right
    text-edit-mode-place-point-mark
    text-edit-mode-place-line-mark
    text-edit-mode-place-box-mark
    text-edit-mode-place-buffer-mark
    text-edit-mode-remove-mark
    text-edit-mode-cursor-create
    text-edit-mode-cursor-destroy
    text-edit-mode-cursor-position
    text-edit-mode-cursor-count
    text-edit-mode-main-cursor-index
    text-edit-mode-is-mark-set?
    text-edit-mode-insert-at-cursor
    text-edit-mode-delete-before-cursor
    text-edit-mode-delete-after-cursor
    text-edit-mode-delete-cursor-region
    text-edit-mode-replace-at-cursor
    text-edit-mode-undo
    text-edit-mode-redo
    text-edit-mode-start-transaction
    text-edit-mode-end-transaction
    text-edit-mode-insert-key
    text-edit-mode-file-open-hook)
  (import (rnrs)
    (major-mode)
    (koru-command)
    (koru-session)
    (koru-buffer)
    (styled-text)
    (text-edit))

  (create-hook 'text-edit-mode)

  (define text-edit-mode-cursor-up
    (command-create
      'text-edit-mode-cursor-up
      "Moves the cursor at the index up"
      (lambda (index) (text-edit-move-cursor-up (current-major-mode) index))
      'number))

  (define text-edit-mode-cursor-down
    (command-create
      'text-edit-mode-cursor-down
      "Moves the cursor at the index down"
      (lambda (index) (text-edit-move-cursor-down (current-major-mode) index))
      'number))

  (define text-edit-mode-cursor-left
    (command-create
      'text-edit-mode-cursor-left
      "Moves the cursor at the index left with the possiblity of wrapping at the start"
      (lambda (index wrap) (text-edit-move-cursor-left (current-major-mode) index wrap))
      'number
      'boolean))

  (define text-edit-mode-cursor-right
    (command-create
      'text-edit-mode-cursor-right
      "Moves the cursor at the index right with the possiblity of wrapping at the end"
      (lambda (index wrap) (text-edit-move-cursor-right (current-major-mode) index wrap))
      'number))

    (define text-edit-mode-place-point-mark
      (command-create
        'text-edit-mode-place-point-mark
        "Places a point mark at the position of the cursor indicated by the index"
        (lambda (index) (text-edit-place-point-mark-at-cursor (current-major-mode) index))
        'number))

  (define text-edit-mode-place-line-mark
    (command-create
      'text-edit-mode-place-line-mark
      "Places a line mark at the position of the cursor indicated by the index"
      (lambda (index) (text-edit-place-line-mark-at-cursor (current-major-mode) index))
      'number))

  (define text-edit-mode-place-box-mark
    (command-create
      'text-edit-mode-place-box-mark
      "Places a box mark at the position of the cursor indicated by the index"
      (lambda (index) (text-edit-place-box-mark-at-cursor (current-major-mode) index))
      'number))

  (define text-edit-mode-place-buffer-mark
    (command-create
      'text-edit-mode-place-buffer-mark
      "Selects the whole file"
      (lambda (index) (text-edit-place-buffer-mark-at-cursor (current-major-mode) index))
      'number))

    (define text-edit-mode-remove-mark
      (command-create
        'text-edit-mode-remove-mark
        "Removes a mark of the cursor indicated by the index"
        (lambda (index) (text-edit-remove-mark-from-cursor (current-major-mode) index))
        'number))

    (define text-edit-mode-cursor-create
      (command-create
        'text-edit-mode-cursor-create
        "Creates a new cursor at the indicated row and column"
        (lambda (row column) (text-edit-cursor-create (current-major-mode) row column))
        'number
        'number))

  (define text-edit-mode-cursor-destroy
    (command-create
      'text-edit-mode-cursor-destroy
      "Removes the cursor at the indicated index"
      (lambda (index) (text-edit-cursor-destroy (current-major-mode) index))
      'number))

    (define (text-edit-mode-cursor-position index)
      (text-edit-cursor-position (current-major-mode) index))

    (define (text-edit-mode-cursor-count)
      (text-edit-cursor-count (current-major-mode)))

  (define (text-edit-mode-main-cursor-index)
    (text-edit-get-main-cursor-index (current-major-mode)))

  (define (text-edit-mode-is-mark-set? index)
    (text-edit-is-mark-set? (current-major-mode) index))

  (define text-edit-mode-cursor-main
    (command-create
      'text-edit-mode-cursor-main
      "Makes the cursor at the indicated index the new main cursor"
      (lambda (index) (text-edit-cursor-change-main (current-major-mode) index))
      'number))

  (define text-edit-mode-insert-at-cursor
    (command-create
      'text-edit-mode-insert-at-cursor
      "Inserts the text at the indicated cursor index"
      (lambda (index text) (text-edit-insert-at-cursor (current-major-mode) index text))
      'number
      'text))

  (define text-edit-mode-delete-before-cursor
    (command-create
      'text-edit-mode-delete-before-cursor
      "Deletes a character before the cursor"
      (lambda (index) (text-edit-delete-before-cursor (current-major-mode) index))
      'number))

  (define text-edit-mode-delete-after-cursor
    (command-create
      'text-edit-mode-delete-after-cursor
      "Deletes a character after the cursor"
      (lambda (index) (text-edit-delete-after-cursor (current-major-mode) index))
      'number))

  (define text-edit-mode-delete-cursor-region
    (command-create
      'text-edit-mode-delete-cursor-region
      "Deletes text within marked area under cursor"
      (lambda (index) (text-edit-delete-region-cursor (current-major-mode) index))
      'number))

  (define text-edit-mode-replace-at-cursor
    (command-create
      'text-edit-mode-replace-at-cursor
      "Inserts the text over the region or character at the indicated cursor index"
      (lambda (index text) (text-edit-replace-text (current-major-mode) index text))
      'number
      'text))

  (define text-edit-mode-undo
    (command-create
      'text-edit-mode-undo
      "Undoes a text modification action"
      (lambda () (text-edit-undo (current-major-mode)))))

  (define text-edit-mode-redo
    (command-create
      'text-edit-mode-redo
      "Redoes a text modification action"
      (lambda () (text-edit-redo (current-major-mode)))))

  (define text-edit-mode-start-transaction
    (command-create
      'text-edit-mode-start-transaction
      "Starts an undo/redo transaction"
      (lambda () (text-edit-start-transaction (current-major-mode)))))

  (define text-edit-mode-end-transaction
    (command-create
      'text-edit-mode-end-transaction
      "Ends an undo/redo transaction"
      (lambda () (text-edit-end-transaction (current-major-mode)))))

  (define text-edit-mode-insert-key
    (command-create
      'text-edit-mode-insert-key
      "Inserts a key if the key sequence when the length is 1"
      (lambda (cursor-index key-seq) (text-edit-insert-keypress (current-major-mode) cursor-index key-seq))
      'number
      'key-sequence))

  (define (text-edit-mode-create buffer-name)
    (major-mode-create
      'TextEdit
      (lambda (major-mode)
        (let ((data (major-mode-data major-mode)))
          (plain-draw (text-edit-get-buffer-name data) (text-edit-get-cursors data))))
      (lambda (major-mode)
        (text-edit-get-main-cursor major-mode))
      (lambda (major-mode) '())
      (lambda (major-mode) '())
      (text-edit-data-create buffer-name)))

  (define (text-edit-mode-file-open-hook buffer-name file-ext)
    (major-mode-set! buffer-name (text-edit-mode-create buffer-name))
    (emit-hook 'text-edit-mode)))

