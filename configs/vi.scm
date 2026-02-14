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
        (command-apply editor-remove-mark)
        (vi-state-set! (minor-mode-get 'vi-mode) 'Normal))
      #t
      'key-sequence))

  (define command-insert-text
    (command-create
      'command-insert-text
      "Inserts text into the command bar from a key sequence"
      (lambda (keys) (command-bar-insert-key keys)
                     (command-bar-update (vi-prefix (minor-mode-get 'vi-mode))))
      #t
      'key-sequence))

  (define command-insert-space
    (command-create
      'command-insert-text
      "Inserts text into the command bar from a key sequence"
      (lambda (keys) (command-bar-insert " ")
                     (command-bar-update (vi-prefix (minor-mode-get 'vi-mode))))
      #t
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
      #t
      'key-sequence))

  (define command-delete-back
    (command-create
      'command-delete-back
      "Deletes backwards in the command bar"
      (lambda (keys)
        (command-bar-delete-back)
        (command-bar-update (vi-prefix (minor-mode-get 'vi-mode))))
      #t
      'key-sequence))

  (define command-delete-forward
    (command-create
      'command-delete-forward
      "Deletes forwards in the command bar"
      (lambda (keys)
        (command-bar-delete-forward)
        (command-bar-update (vi-prefix (minor-mode-get 'vi-mode))))
      #t
      'key-sequence))

  (define command-cursor-left
    (command-create
      'command-cursor-left
      "Moves the cursor to the left in the command bar"
      (lambda (keys)
        (command-bar-left)
        (command-bar-update (vi-prefix (minor-mode-get 'vi-mode))))
      #t
      'key-sequence))

  (define command-cursor-right
    (command-create
      'command-cursor-right
      "Moves the cursor to the right in the command bar"
      (lambda (keys)
        (command-bar-right)
        (command-bar-update (vi-prefix (minor-mode-get 'vi-mode)))
        #t
      'key-sequence)))

  (define vi-enter-insert-keypress
    (command-create
      'vi-enter-insert
      "Enters into insert mode"
      (lambda (keys) (vi-state-set! (minor-mode-get 'vi-mode) 'Insert))
      #t
      'key-sequence))

  (define vi-enter-visual-keypress
    (command-create
      'vi-enter-visual-keypress
      "Enters into visual mode with a point selection."
      (lambda (keys)
        (command-apply text-edit-mode-place-point-mark 0)
        (vi-state-set! (minor-mode-get 'vi-mode) 'Visual))
      #t
      'key-sequence))

  (define vi-enter-visual-box-keypress
    (command-create
      'vi-enter-visual-box-keypress
      "Enters into visual mode with a box selection."
      (lambda (keys)
        (command-apply text-edit-mode-place-box-mark 0)
        (vi-state-set! (minor-mode-get 'vi-mode) 'Visual))
      #t
      'key-sequence))

  (define vi-enter-visual-line-keypress
    (command-create
      'vi-enter-visual-line-keypress
      "Enters into visual mode with a line selection."
      (lambda (keys)
        (command-apply text-edit-mode-place-line-mark 0)
        (vi-state-set! (minor-mode-get 'vi-mode) 'Visual))
      #t
      'key-sequence))

  (define vi-visual-visual-delete-keypress
    (command-create
      'vi-visual-visual-delete-keypress
      "Deletes the highlighted region and enters into Normal mode."
      (lambda (keys)
        (command-apply text-edit-mode-delete-cursor-region 0)
        (command-apply editor-remove-mark)
        (vi-state-set! (minor-mode-get 'vi-mode) 'Normal))
      #t
      'key-sequence))

  (define vi-enter-command-keypress
    (command-create
      'vi-enter-command
      "Enters into command mode"
      (lambda (keys)
        (vi-callback-set! (minor-mode-get 'vi-mode)
          (lambda () (command-bar-take)))
        (vi-prefix-set! (minor-mode-get 'vi-mode) ":")
        (command-bar-show)
        (command-bar-update (vi-prefix (minor-mode-get 'vi-mode)))
        (vi-state-set! (minor-mode-get 'vi-mode) 'Command))
      #t
      'key-sequence))

  (define (vi-normal-mode-keymap)
    (let ((vi-key-map (key-map-create)))
      (key-map-insert vi-key-map "UP" editor-cursor-up-keypress)
      (key-map-insert vi-key-map "k" editor-cursor-up-keypress)
      (key-map-insert vi-key-map "DOWN" editor-cursor-down-keypress)
      (key-map-insert vi-key-map "j" editor-cursor-down-keypress)
      (key-map-insert vi-key-map "LEFT" editor-cursor-left-keypress)
      (key-map-insert vi-key-map "h" editor-cursor-left-keypress)
      (key-map-insert vi-key-map "RIGHT" editor-cursor-right-keypress)
      (key-map-insert vi-key-map "l" editor-cursor-right-keypress)
      (key-map-insert vi-key-map "i" vi-enter-insert-keypress)
      (key-map-insert vi-key-map "v" vi-enter-visual-keypress)
      (key-map-insert vi-key-map "V" vi-enter-visual-line-keypress)
      (key-map-insert vi-key-map "C-v" vi-enter-visual-box-keypress)
      (key-map-insert vi-key-map ":" vi-enter-command-keypress)
      (key-map-insert vi-key-map "u" editor-undo-keypress)
      (key-map-insert vi-key-map "C-r" editor-redo-keypress)
      (key-map-insert vi-key-map "C-c" editor-cursor-add-below-keypress)
      (key-map-insert vi-key-map "x" editor-delete-forward-keypress)
      vi-key-map))

  (define (vi-visual-mode-keymap)
    (let ((vi-key-map (key-map-create)))
      (key-map-insert vi-key-map "UP" editor-cursor-up-keypress)
      (key-map-insert vi-key-map "k" editor-cursor-up-keypress)
      (key-map-insert vi-key-map "DOWN" editor-cursor-down-keypress)
      (key-map-insert vi-key-map "j" editor-cursor-down-keypress)
      (key-map-insert vi-key-map "LEFT" editor-cursor-left-keypress)
      (key-map-insert vi-key-map "h" editor-cursor-left-keypress)
      (key-map-insert vi-key-map "RIGHT" editor-cursor-right-keypress)
      (key-map-insert vi-key-map "l" editor-cursor-right-keypress)
      (key-map-insert vi-key-map "x" vi-visual-visual-delete-keypress)
      (key-map-insert vi-key-map "d" vi-visual-visual-delete-keypress)
      vi-key-map))

  (define (vi-insert-mode-keymap)
    (let ((vi-key-map (key-map-create editor-insert-text-keypress)))
      (key-map-insert vi-key-map "UP" editor-cursor-up-keypress)
      (key-map-insert vi-key-map "DOWN" editor-cursor-down-keypress)
      (key-map-insert vi-key-map "LEFT" editor-cursor-left-keypress)
      (key-map-insert vi-key-map "RIGHT" editor-cursor-right-keypress)
      (key-map-insert vi-key-map "BS" editor-delete-back-keypress)
      (key-map-insert vi-key-map "DEL" editor-delete-forward-keypress)
      (key-map-insert vi-key-map "ENTER" editor-insert-newline-keypress)
      (key-map-insert vi-key-map "SPC" editor-insert-space-keypress)
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
    (add-special-key-binding "C-q" editor-crash)
    (create-hook 'vi-mode-change)
    (add-hook 'buffer-open 'text-edit-mode text-edit-mode-file-open-hook)
    (add-hook 'buffer-open 'vi-mode vi-config-hook)))
