(library (configs emacs)
  (export init-emacs-config)
  (import (rnrs)
    (koru-command)
    (koru-session)
    (koru-key)
    (scheme text-edit-mode))

  (define editor-cursor-up
    (command-create
      "editor-cursor-up"
      "Moves the primary cursor up"
      (lambda (keys) (command-apply text-edit-mode-cursor-up 0))
      "key-sequence"))

  (define editor-cursor-down
    (command-create
      "editor-cursor-down"
      "Moves the primary cursor down"
      (lambda (keys) (command-apply text-edit-mode-cursor-down 0))
      "key-sequence"))

  (define editor-cursor-left
    (command-create
      "editor-cursor-left"
      "Moves the primary cursor left"
      (lambda (keys) (command-apply text-edit-mode-cursor-left 0 #f))
      "key-sequence"))

  (define editor-cursor-right
    (command-create
      "editor-cursor-right"
      "Moves the primary cursor right"
      (lambda (keys) (command-apply text-edit-mode-cursor-right 0 #f))
      "key-sequence"))

  (define editor-place-point-mark
    (command-create
      "editor-place-point-mark"
      "Places a point mark at the primary cursor"
      (lambda (keys) (command-apply text-edit-mode-place-point-mark 0))
      "key-sequence"))

  (define editor-remove-mark
    (command-create
      "editor-remove-mark"
      "Removes the mark at the primary cursor and flushes the keybuffer"
      (lambda (keys) (begin
                       (flush-key-buffer)
                       (command-apply text-edit-mode-remove-mark 0)))
      "key-sequence"))

  (define editor-insert-text
    (command-create
      "editor-insert-text"
      "Inserts text at the primary cursor"
      (lambda (keys) (begin
                       (if (command-apply text-edit-mode-insert-key 0 keys)
                         (begin (command-apply text-edit-mode-cursor-right 0 #f) #t)
                         #f)))
      "key-sequence"))

  (define editor-return
    (command-create
      "editor-return"
      "Inserts a newline at the primary cursor"
      (lambda (keys) (begin
                       (command-apply text-edit-mode-insert-at-cursor 0 "\n")
                       (command-apply text-edit-mode-cursor-right 0 #t)))
      "key-sequence"))

  (define editor-delete-back
    (command-create
      "editor-delete-back"
      "Deletes text before the primary cursor"
      (lambda (keys) (begin
                       (command-apply text-edit-mode-delete-before-cursor 0)
                       (command-apply text-edit-mode-cursor-left 0 #t)
                       ))
      "key-sequence"))

  (define editor-delete-forward
    (command-create
      "editor-delete-forward"
      "Deletes text at the primary cursor"
      (lambda (keys) (command-apply text-edit-mode-delete-after-cursor 0))
      "key-sequence"))

  (define editor-delete-region
    (command-create
      "editor-delete-region"
      "Deletes text in text region of primary cursor"
      (lambda (keys) (command-apply text-edit-mode-delete-cursor-region 0))
      "key-sequence"))

  (define editor-undo
    (command-create
      "editor-undo"
      "Undoes a text modification"
      (lambda (keys) (command-apply text-edit-mode-undo))
      "key-sequence"))

  (define editor-redo
    (command-create
      "editor-redo"
      "Redoes a text modification"
      (lambda (keys) (command-apply text-edit-mode-redo))
      "key-sequence"))

  (define emacs-editor-key-map (key-map-create editor-insert-text))

  (key-map-insert emacs-editor-key-map "UP" editor-cursor-up)
  (key-map-insert emacs-editor-key-map "DOWN" editor-cursor-down)
  (key-map-insert emacs-editor-key-map "LEFT" editor-cursor-left)
  (key-map-insert emacs-editor-key-map "RIGHT" editor-cursor-right)
  (key-map-insert emacs-editor-key-map "C-p" editor-cursor-up)
  (key-map-insert emacs-editor-key-map "C-n" editor-cursor-down)
  (key-map-insert emacs-editor-key-map "C-b" editor-cursor-left)
  (key-map-insert emacs-editor-key-map "C-f" editor-cursor-right)
  (key-map-insert emacs-editor-key-map "BS" editor-delete-back)
  (key-map-insert emacs-editor-key-map "DEL" editor-delete-forward)
  (key-map-insert emacs-editor-key-map "ENTER" editor-return)
  (key-map-insert emacs-editor-key-map "C-SPC" editor-place-point-mark)
  ;(key-map-insert emacs-editor-key-map "C-g" editor-remove-mark)
  (key-map-insert emacs-editor-key-map "C-w" editor-delete-region)
  (key-map-insert emacs-editor-key-map "C-_" editor-undo)
  (key-map-insert emacs-editor-key-map "C-x u" editor-redo)

  (define (emacs-config-hook)
    (add-key-map "emacs-edit" emacs-editor-key-map)
    (add-special-key-binding "C-g" editor-remove-mark))


  (define (init-emacs-config)
    (add-hook "buffer-open" "text-edit-mode" text-edit-mode-file-open-hook)
    (add-hook "text-edit-mode" "emacs-editor-config" emacs-config-hook)))