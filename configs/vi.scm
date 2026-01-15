(library (configs vi)
  (export init-vi-config)
  (import (rnrs)
    (koru-command)
    (koru-session)
    (koru-key)
    (minor-mode)
    (koru-buffer)
    (koru-modal)
    (major-mode)
    (configs common)
    (scheme text-edit-mode))

  (define (vi-state vi-mode)
    (modal-state (minor-mode-data vi-mode)))
  (define (vi-state-set! vi-mode new)
    (modal-state-set! (minor-mode-data vi-mode) new))

  (define (vi-prefix vi-mode)
    (modal-prefix (minor-mode-data vi-mode)))
  (define (vi-prefix-set! vi-mode new)
    (modal-prefix-set! (minor-mode-data vi-mode) new))

  (define (vi-callback-set! vi-mode new)
    (modal-callback-set! (minor-mode-data vi-mode) new))
  (define (vi-callback-apply vi-mode)
    (modal-callback-apply (minor-mode-data vi-mode)))

  (define editor-crash
    (command-create
      'editor-crash
      "Crashes the editor"
      (lambda (keys) (crash))
      'key-sequence))

  (define vi-escape
    (command-create
      'vi-escape
      "Clears the keybuffer and leaves the current mode if it isn't Normal mode"
      (lambda (keys)
        (flush-key-buffer)
        (command-bar-take)
        (command-bar-update)
        (command-bar-hide)
        (command-apply text-edit-mode-remove-mark 0)
        (vi-state-set! (minor-mode-get 'vi-mode) 'Normal))
      'key-sequence))

  (define command-insert-text
    (command-create
      'command-insert-text
      "Inserts text into the command bar from a key sequence"
      (lambda (keys) (command-bar-insert-key keys)
                     (command-bar-update (vi-prefix (minor-mode-get 'vi-mode))))
      'key-sequence))

  (define command-insert-space
    (command-create
      'command-insert-text
      "Inserts text into the command bar from a key sequence"
      (lambda (keys) (command-bar-insert " ")
                     (command-bar-update (vi-prefix (minor-mode-get 'vi-mode))))
      'key-sequence))

  (define command-activate
    (command-create
      'command-activate
      "Activates the command"
      (lambda (keys)
        (vi-callback-apply (minor-mode-get 'vi-mode))
        (command-bar-update)
        (command-bar-hide)
        (vi-state-set! (minor-mode-get 'vi-mode) 'Normal))
      'key-sequence))

  (define command-delete-back
    (command-create
      'command-delete-back
      "Deletes backwards in the command bar"
      (lambda (keys)
        (command-bar-delete-back)
        (command-bar-update (vi-prefix (minor-mode-get 'vi-mode))))
      'key-sequence))

  (define command-delete-forward
    (command-create
      'command-delete-forward
      "Deletes forwards in the command bar"
      (lambda (keys)
        (command-bar-delete-forward)
        (command-bar-update (vi-prefix (minor-mode-get 'vi-mode))))
      'key-sequence))

  (define command-cursor-left
    (command-create
      'command-cursor-left
      "Moves the cursor to the left in the command bar"
      (lambda (keys)
        (command-bar-left)
        (command-bar-update (vi-prefix (minor-mode-get 'vi-mode))))
      'key-sequence))

  (define command-cursor-right
    (command-create
      'command-cursor-right
      "Moves the cursor to the right in the command bar"
      (lambda (keys)
        (command-bar-right)
        (command-bar-update (vi-prefix (minor-mode-get 'vi-mode)))
      'key-sequence)))

  (define vi-enter-insert
    (command-create
      'vi-enter-insert
      "Enters into insert mode"
      (lambda (keys) (vi-state-set! (minor-mode-get 'vi-mode) 'Insert))
      'key-sequence))

  (define vi-enter-visual
    (command-create
      'vi-enter-visual
      "Enters into visual mode"
      (lambda (keys)
        (command-apply text-edit-mode-place-point-mark 0)
        (vi-state-set! (minor-mode-get 'vi-mode) 'Visual))
      'key-sequence))

  (define vi-enter-command
    (command-create
      'vi-enter-command
      "Enters into command mode"
      (lambda (keys)
        (display "setting callback\n")
        (vi-callback-set! (minor-mode-get 'vi-mode)
          (lambda () (command-bar-take)))
        (vi-prefix-set! (minor-mode-get 'vi-mode) ":")
        (command-bar-show)
        (command-bar-update (vi-prefix (minor-mode-get 'vi-mode)))
        (vi-state-set! (minor-mode-get 'vi-mode) 'Command))
      'key-sequence))


  (define cursor-add
    (command-create
      'cursor-add
      "Adds a cursor below the last cursor"
      (lambda (keys) (let ((last-cursor (- (text-edit-mode-cursor-count) 1)))
                      (let ((point (text-edit-mode-cursor-position last-cursor)))
                        (command-apply text-edit-mode-cursor-create (+ (car point) 1) (cdr point)))))
      'key-sequence))

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
      (key-map-insert vi-key-map "u" editor-undo)
      (key-map-insert vi-key-map "C-r" editor-redo)
      (key-map-insert vi-key-map "C-q" cursor-add)
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
      (key-map-insert vi-key-map "SPC" editor-insert-space)
      vi-key-map))

  (define (vi-command-mode-keymap)
    (let ((vi-key-map (key-map-create command-insert-text)))
      (key-map-insert vi-key-map "LEFT" command-cursor-left)
      (key-map-insert vi-key-map "RIGHT" command-cursor-right)
      (key-map-insert vi-key-map "BS" command-delete-back)
      (key-map-insert vi-key-map "DEL" command-delete-forward)
      (key-map-insert vi-key-map "ENTER" command-activate)
      (key-map-insert vi-key-map "SPC" command-insert-space)
      vi-key-map))

  (define (enter-normal-mode)
    (when (is-current-buffer-set?)
      (command-apply text-edit-mode-end-transaction))
    (add-key-map 'vi-edit (vi-normal-mode-keymap)))

  (define (enter-normal-mode-first-time)
    (add-key-map 'vi-edit (vi-normal-mode-keymap)))

  (define (enter-insert-mode)
    (when (is-current-buffer-set?)
      (command-apply text-edit-mode-start-transaction))
    (add-key-map 'vi-edit (vi-insert-mode-keymap)))

  (define (enter-visual-mode)
    (when (is-current-buffer-set?)
      (command-apply text-edit-mode-start-transaction))
    (add-key-map 'vi-edit (vi-visual-mode-keymap)))

  (define (enter-command-mode)
    (when (is-current-buffer-set?)
      (command-apply text-edit-mode-start-transaction))
    (add-key-map 'vi-edit (vi-command-mode-keymap)))

  (define (vi-enter-mode vi-mode mode)
    (remove-key-map 'vi-edit)
    (cond
      ((equal? mode 'Normal) (enter-normal-mode))
      ((equal? mode 'Insert) (enter-insert-mode))
      ((equal? mode 'Visual) (enter-visual-mode))
      ((equal? mode 'Command) (enter-command-mode))))


  (define (vi-gain-focus vi-mode)
    (vi-enter-mode vi-mode (vi-state vi-mode))
    (add-special-key-binding "ESC" vi-escape))
  (define (vi-lose-focus vi-mode)
    (remove-key-map 'vi-edit)
    (remove-special-key-binding "ESC"))

  (define (vi-mode)
    (let ((vi-mode (minor-mode-create 'vi-mode vi-gain-focus vi-lose-focus)))
      (let ((vi-modal (modal-create 'Normal 'vi-mode-change (lambda (old new) (vi-enter-mode vi-mode new)))))
        (minor-mode-data-set! vi-mode vi-modal)
        vi-mode)))

  (define (vi-config-hook buffer-name file-ext)
    (minor-mode-add buffer-name (vi-mode))
    (enter-normal-mode-first-time))

  (define (init-vi-config)
    (add-special-key-binding "C-c" editor-crash)
    (create-hook 'vi-mode-change)
    (add-hook 'buffer-open 'text-edit-mode text-edit-mode-file-open-hook)
    (add-hook 'buffer-open 'vi-mode vi-config-hook)))
