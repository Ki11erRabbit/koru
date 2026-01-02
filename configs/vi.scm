(library (configs emacs)
  (export init-vi-config)
  (import (rnrs)
    (koru-command)
    (koru-session)
    (koru-key)
    (minor-mode)
    (koru-buffer)
    (major-mode)
    (scheme text-edit-mode))

  (define vi-escape
    (command-create
      "vi-escape"
      "Clears the keybuffer and leaves the current mode if it isn't Normal mode"
      (lambda (keys)
        (flush-key-buffer)
        (command-bar-take)
        (command-bar-update)
        (command-bar-hide)
        (vi-change-mode "Normal"))
      "key-sequence"))

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

  (define editor-insert-text
    (command-create
      "editor-insert-text"
      "Inserts text at the primary cursor"
      (lambda (keys) (if (command-apply text-edit-mode-insert-key 0 keys)
                       (begin (command-apply text-edit-mode-cursor-right 0 #f) #t)
                       #f))
      "key-sequence"))

  (define command-insert-text
    (command-create
      "command-insert-text"
      "Inserts text into the command bar from a key sequence"
      (lambda (keys) (command-bar-insert-key keys)
                     (command-bar-update))
      "key-sequence"))

  (define command-activate
    (command-create
      "command-activate"
      "Activates the command"
      (lambda (keys) (command-bar-take)
                     (command-bar-update)
                     (command-bar-hide)
                     (vi-change-mode "Normal"))
      "key-sequence"))

  (define command-delete-back
    (command-create
      "command-delete-back"
      "Deletes backwards in the command bar"
      (lambda (keys) (command-bar-delete-back))
      "key-sequence"))

  (define command-delete-forward
    (command-create
      "command-delete-forward"
      "Deletes forwards in the command bar"
      (lambda (keys) (command-bar-delete-forward))
      "key-sequence"))

  (define command-cursor-left
    (command-create
      "command-cursor-left"
      "Moves the cursor to the left in the command bar"
      (lambda (keys) (command-bar-left))
      "key-sequence"))

  (define command-cursor-right
    (command-create
      "command-cursor-right"
      "Moves the cursor to the right in the command bar"
      (lambda (keys) (command-bar-right))
      "key-sequence"))

  (define vi-enter-insert
    (command-create
      "vi-enter-insert"
      "Enters into insert mode"
      (lambda (keys) (vi-change-mode "Insert"))
      "key-sequence"))

  (define vi-enter-visual
    (command-create
      "vi-enter-visual"
      "Enters into visual mode"
      (lambda (keys) (vi-change-mode "Visual"))
      "key-sequence"))

  (define vi-enter-command
    (command-create
      "vi-enter-command"
      "Enters into command mode"
      (lambda (keys) (vi-change-mode "Command"))
      "key-sequence"))

  (define (vi-normal-mode-keymap)
    (let ((vi-key-map (key-map-create)))
      (key-map-insert vi-key-map "UP" editor-cursor-up)
      (key-map-insert vi-key-map "k" editor-cursor-up)
      (key-map-insert vi-key-map "DOWN" editor-cursor-down)
      (key-map-insert vi-key-map "j" editor-cursor-down)
      (key-map-insert vi-key-map "LEFT" editor-cursor-left)
      (key-map-insert vi-key-map "h" editor-cursor-left)
      (key-map-insert vi-key-map "RIGHT" editor-cursor-right)
      (key-map-insert vi-key-map "l" editor-cursor-right)
      (key-map-insert vi-key-map "i" vi-enter-insert)
      (key-map-insert vi-key-map "v" vi-enter-visual)
      (key-map-insert vi-key-map ":" vi-enter-command)
      vi-key-map))

  (define (vi-visual-mode-keymap)
    (let ((vi-key-map (key-map-create)))
      (key-map-insert vi-key-map "UP" editor-cursor-up)
      (key-map-insert vi-key-map "k" editor-cursor-up)
      (key-map-insert vi-key-map "DOWN" editor-cursor-down)
      (key-map-insert vi-key-map "j" editor-cursor-down)
      (key-map-insert vi-key-map "LEFT" editor-cursor-left)
      (key-map-insert vi-key-map "h" editor-cursor-left)
      (key-map-insert vi-key-map "RIGHT" editor-cursor-right)
      (key-map-insert vi-key-map "l" editor-cursor-right)
      vi-key-map))

  (define (vi-insert-mode-keymap)
    (let ((vi-key-map (key-map-create editor-insert-text)))
      (key-map-insert vi-key-map "UP" editor-cursor-up)
      (key-map-insert vi-key-map "DOWN" editor-cursor-down)
      (key-map-insert vi-key-map "LEFT" editor-cursor-left)
      (key-map-insert vi-key-map "RIGHT" editor-cursor-right)
      (key-map-insert vi-key-map "BS" editor-delete-back)
      (key-map-insert vi-key-map "DEL" editor-delete-forward)
      (key-map-insert vi-key-map "ENTER" editor-return)
      vi-key-map))

  (define (vi-command-mode-keymap)
    (let ((vi-key-map (key-map-create command-insert-text)))
      (key-map-insert vi-key-map "LEFT" command-cursor-left)
      (key-map-insert vi-key-map "RIGHT" command-cursor-right)
      (key-map-insert vi-key-map "BS" command-delete-back)
      (key-map-insert vi-key-map "DEL" command-delete-forward)
      (key-map-insert vi-key-map "ENTER" command-activate)
      vi-key-map))

  (define (enter-normal-mode)
    (command-apply text-edit-mode-remove-mark 0)
    (add-key-map "vi-edit" (vi-normal-mode-keymap)))

  (define (enter-normal-mode-first-time)
    (add-key-map "vi-edit" (vi-normal-mode-keymap)))

  (define (enter-insert-mode)
    (add-key-map "vi-edit" (vi-insert-mode-keymap)))

  (define (enter-visual-mode)
    (command-apply text-edit-mode-place-point-mark 0)
    (add-key-map "vi-edit" (vi-visual-mode-keymap)))

  (define (enter-command-mode)
    (command-bar-show)
    (add-key-map "vi-edit" (vi-command-mode-keymap)))

  (define (vi-enter-mode mode)
    (cond
      ((equal? mode "Normal") (enter-normal-mode))
      ((equal? mode "Insert") (enter-insert-mode))
      ((equal? mode "Visual") (enter-visual-mode))
      ((equal? mode "Command") (enter-command-mode))))

  (define (vi-change-keymap mode)
    (remove-key-map "vi-edit")
    (vi-enter-mode mode))

  (define (vi-change-mode-internal vi-mode mode)
    (if (not (equal? (minor-mode-data vi-mode) mode))
      (begin
        (emit-hook "vi-mode-change" (minor-mode-data vi-mode) mode)
        (vi-change-keymap mode)
        (minor-mode-data-set! vi-mode mode))
      '()))

  (define (vi-change-mode mode)
    (let ((vi-mode (minor-mode-get "vi-mode")))
      (vi-change-mode-internal vi-mode mode)))

  (define (vi-gain-focus vi-mode)
    (add-special-key-binding "ESC" vi-escape)
    (vi-enter-mode (minor-mode-data vi-mode)))
  (define (vi-lose-focus vi-mode)
    (remove-special-key-binding "ESC")
    (remove-key-map "vi-edit"))

  (define (vi-mode)
    (minor-mode-create "vi-mode" vi-gain-focus vi-lose-focus ""))

  (define (vi-config-hook buffer-name file-ext)
    (let ((mode (vi-mode)))
      (minor-mode-add buffer-name mode)
      (minor-mode-data-set! mode "Normal")
      (enter-normal-mode-first-time)))

  (define (init-vi-config)
    (create-hook "vi-mode-change")
    (add-hook "buffer-open" "text-edit-mode" text-edit-mode-file-open-hook)
    (add-hook "buffer-open" "vi-mode" vi-config-hook)))
