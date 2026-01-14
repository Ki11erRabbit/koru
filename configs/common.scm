(library (configs common)
  (export
    editor-cursor-up
    editor-cursor-down
    editor-cursor-left
    editor-cursor-right
    editor-insert-text
    editor-insert-space
    editor-return
    editor-delete-back
    editor-delete-forward
    editor-delete-region
    editor-undo
    editor-redo
    mode-state-create
    mode-state-state
    mode-state-state-change
    mode-state-command-prefix
    mode-state-command-prefix-change
    mode-state-command-suffix
    mode-state-command-suffix-change
    mode-state-command-callback
    mode-state-command-callback-change
    mode-state-data
    mode-state-data-change)
  (import (rnrs)
    (koru-command)
    (koru-session)
    (koru-key)
    (minor-mode)
    (koru-buffer)
    (scheme text-edit-mode))


  (define editor-cursor-up
    (command-create
      'editor-cursor-up
      "Moves the primary cursor up"
      (lambda (keys) (command-apply text-edit-mode-cursor-up 0))
      'key-sequence))

  (define editor-cursor-down
    (command-create
      'editor-cursor-down
      "Moves the primary cursor down"
      (lambda (keys) (command-apply text-edit-mode-cursor-down 0))
      'key-sequence))

  (define editor-cursor-left
    (command-create
      'editor-cursor-left
      "Moves the primary cursor left"
      (lambda (keys) (command-apply text-edit-mode-cursor-left 0 #f))
      'key-sequence))

  (define editor-cursor-right
    (command-create
      'editor-cursor-right
      "Moves the primary cursor right"
      (lambda (keys) (command-apply text-edit-mode-cursor-right 0 #f))
      'key-sequence))

  (define editor-insert-text
    (command-create
      'editor-insert-text
      "Inserts text at the primary cursor"
      (lambda (keys) (begin
                       (if (command-apply text-edit-mode-insert-key 0 keys)
                         (begin (command-apply text-edit-mode-cursor-right 0 #f) #t)
                         #f)))
      'key-sequence))

  (define editor-insert-space
    (command-create
      'editor-insert-text
      "Inserts text at the primary cursor"
      (lambda (keys)
        (command-apply text-edit-mode-insert-at-cursor 0 " ")
        (command-apply text-edit-mode-cursor-right 0 #f))
      'key-sequence))

  (define editor-return
    (command-create
      'editor-return
      "Inserts a newline at the primary cursor"
      (lambda (keys) (begin
                       (command-apply text-edit-mode-insert-at-cursor 0 "\n")
                       (command-apply text-edit-mode-cursor-right 0 #t)))
      'key-sequence))

  (define editor-delete-back
    (command-create
      'editor-delete-back
      "Deletes text before the primary cursor"
      (lambda (keys) (begin
                       (command-apply text-edit-mode-delete-before-cursor 0)))
      'key-sequence))

  (define editor-delete-forward
    (command-create
      'editor-delete-forward
      "Deletes text at the primary cursor"
      (lambda (keys) (command-apply text-edit-mode-delete-after-cursor 0))
      'key-sequence))

  (define editor-delete-region
    (command-create
      'editor-delete-region
      "Deletes text in text region of primary cursor"
      (lambda (keys) (command-apply text-edit-mode-delete-cursor-region 0))
      'key-sequence))

  (define editor-undo
    (command-create
      'editor-undo
      "Undoes a text modification"
      (lambda (keys) (command-apply text-edit-mode-undo))
      'key-sequence))

  (define editor-redo
    (command-create
      'editor-redo
      "Redoes a text modification"
      (lambda (keys) (command-apply text-edit-mode-redo))
      'key-sequence)))