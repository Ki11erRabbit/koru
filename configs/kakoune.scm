(library (configs kakoune)
  (export init-kakoune-config)
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

  (define (kakoune-state kakoune-mode)
    (modal-state (minor-mode-data kakoune-mode)))
  (define (kakoune-state-set! kakoune-mode new)
    (modal-state-set! (minor-mode-data kakoune-mode) new))

  (define (kakoune-prefix kakoune-mode)
    (modal-prefix (minor-mode-data kakoune-mode)))
  (define (kakoune-prefix-set! kakoune-mode new)
    (modal-prefix-set! (minor-mode-data kakoune-mode) new))

  (define (kakoune-callback-set! kakoune-mode new)
    (modal-callback-set! (minor-mode-data kakoune-mode) new))
  (define (kakoune-callback-apply kakoune-mode)
    (modal-callback-apply (minor-mode-data kakoune-mode)))

  (define editor-crash
    (command-create
      'editor-crash
      "Crashes the editor"
      (lambda (keys) (crash))
      'key-sequence))

  (define kakoune-escape
    (command-create
      'kakoune-escape
      "Clears the keybuffer and leaves the current mode if it isn't Normal mode"
      (lambda (keys)
        (flush-key-buffer)
        (command-bar-take)
        (command-bar-update)
        (command-bar-hide)
        (command-apply editor-remove-mark)
        (kakoune-state-set! (minor-mode-get 'kakoune-mode) 'Normal))
      #t
      'key-sequence))

  (define command-insert-text
    (command-create
      'command-insert-text
      "Inserts text into the command bar from a key sequence"
      (lambda (keys) (command-bar-insert-key keys)
                     (command-bar-update (kakoune-prefix (minor-mode-get 'kakoune-mode))))
      #t
      'key-sequence))

  (define command-insert-space
    (command-create
      'command-insert-text
      "Inserts text into the command bar from a key sequence"
      (lambda (keys) (command-bar-insert " ")
                     (command-bar-update (kakoune-prefix (minor-mode-get 'kakoune-mode))))
      #t
      'key-sequence))

  (define command-activate
    (command-create
      'command-activate
      "Activates the command"
      (lambda (keys)
        (kakoune-callback-apply (minor-mode-get 'kakoune-mode))
        (command-bar-update)
        (command-bar-hide)
        (kakoune-state-set! (minor-mode-get 'kakoune-mode) 'Normal))
      #t
      'key-sequence))

  (define command-delete-back
    (command-create
      'command-delete-back
      "Deletes backwards in the command bar"
      (lambda (keys)
        (command-bar-delete-back)
        (command-bar-update (kakoune-prefix (minor-mode-get 'kakoune-mode))))
      #t
      'key-sequence))

  (define command-delete-forward
    (command-create
      'command-delete-forward
      "Deletes forwards in the command bar"
      (lambda (keys)
        (command-bar-delete-forward)
        (command-bar-update (kakoune-prefix (minor-mode-get 'kakoune-mode))))
      #t
      'key-sequence))

  (define command-cursor-left
    (command-create
      'command-cursor-left
      "Moves the cursor to the left in the command bar"
      (lambda (keys)
        (command-bar-left)
        (command-bar-update (kakoune-prefix (minor-mode-get 'kakoune-mode))))
      #t
      'key-sequence))

  (define command-cursor-right
    (command-create
      'command-cursor-right
      "Moves the cursor to the right in the command bar"
      (lambda (keys)
        (command-bar-right)
        (command-bar-update (kakoune-prefix (minor-mode-get 'kakoune-mode)))
        #t
        'key-sequence)))

  (define kakoune-enter-insert-keypress
    (command-create
      'kakoune-enter-insert
      "Enters into insert mode"
      (lambda (keys) (kakoune-state-set! (minor-mode-get 'kakoune-mode) 'Insert))
      #t
      'key-sequence))

  (define kakoune-enter-command-keypress
    (command-create
      'kakoune-enter-command
      "Enters into command mode"
      (lambda (keys)
        (kakoune-callback-set! (minor-mode-get 'kakoune-mode)
          (lambda () (command-bar-take)))
        (kakoune-prefix-set! (minor-mode-get 'kakoune-mode) ":")
        (command-bar-show)
        (command-bar-update (kakoune-prefix (minor-mode-get 'kakoune-mode)))
        (kakoune-state-set! (minor-mode-get 'kakoune-mode) 'Command))
      #t
      'key-sequence))

  (define kak-move-cursor-up
    (command-create
      'kak-move-cursor-up
      "Move the cursors up. If there is a selection, then it is ended"
      (lambda () (when (text-edit-mode-is-mark-set? 0)
                   (command-apply editor-remove-mark))
                 (command-apply editor-cursor-up))))

  (define kak-move-cursor-up-keypress
    (command-create
      'kak-move-cursor-up-keypress
      "Move the cursors up. If there is a selection, then it is ended. Takes a keypress."
      (lambda (keys) (command-apply kak-move-cursor-up))
      #t
      'key-sequence))

  (define kak-extend-cursor-up
    (command-create
      'kak-extend-cursor-up
      "Move the cursors up and extend the selection"
      (lambda () (when (not (text-edit-mode-is-mark-set? 0))
                   (command-apply editor-place-point-mark))
                 (command-apply editor-cursor-up))))

  (define kak-extend-cursor-up-keypress
    (command-create
      'kak-extend-cursor-up-keypress
      "Move the cursors up and extend the selection. Takes a keypress."
      (lambda (keys) (command-apply kak-extend-cursor-up))
      #t
      'key-sequence))

  (define kak-move-cursor-down
    (command-create
      'kak-move-cursor-down
      "Move the cursors down. If there is a selection, then it is ended"
      (lambda () (when (text-edit-mode-is-mark-set? 0)
                   (command-apply editor-remove-mark))
                 (command-apply editor-cursor-down))))

  (define kak-move-cursor-down-keypress
    (command-create
      'kak-move-cursor-down-keypress
      "Move the cursors down. If there is a selection, then it is ended. Takes a keypress."
      (lambda (keys) (command-apply kak-move-cursor-down))
      #t
      'key-sequence))

  (define kak-extend-cursor-down
    (command-create
      'kak-extend-cursor-down
      "Move the cursors down and extend the selection"
      (lambda () (when (not (text-edit-mode-is-mark-set? 0))
                   (command-apply editor-place-point-mark))
                 (command-apply editor-cursor-down))))

  (define kak-extend-cursor-down-keypress
    (command-create
      'kak-extend-cursor-down-keypress
      "Move the cursors down and extend the selection. Takes a keypress."
      (lambda (keys) (command-apply kak-extend-cursor-down))
      #t
      'key-sequence))

  (define kak-move-cursor-left
    (command-create
      'kak-move-cursor-left
      "Move the cursors left. If there is a selection, then it is ended"
      (lambda () (when (text-edit-mode-is-mark-set? 0)
                   (command-apply editor-remove-mark))
                 (command-apply editor-cursor-left #f))))

  (define kak-move-cursor-left-keypress
    (command-create
      'kak-move-cursor-left-keypress
      "Move the cursors left. If there is a selection, then it is ended. Takes a keypress."
      (lambda (keys) (command-apply kak-move-cursor-left))
      #t
      'key-sequence))

  (define kak-extend-cursor-left
    (command-create
      'kak-extend-cursor-left
      "Move the cursors left and extend the selection"
      (lambda () (when (not (text-edit-mode-is-mark-set? 0))
                   (command-apply editor-place-point-mark))
                 (command-apply editor-cursor-left #f))))

  (define kak-extend-cursor-left-keypress
    (command-create
      'kak-extend-cursor-left-keypress
      "Move the cursors left and extend the selection. Takes a keypress."
      (lambda (keys) (command-apply kak-extend-cursor-left))
      #t
      'key-sequence))

  (define kak-move-cursor-right
    (command-create
      'kak-move-cursor-right
      "Move the cursors right. If there is a selection, then it is ended"
      (lambda () (when (text-edit-mode-is-mark-set? 0)
                   (command-apply editor-remove-mark))
                 (command-apply editor-cursor-right #f))))

  (define kak-move-cursor-right-keypress
    (command-create
      'kak-move-cursor-right-keypress
      "Move the cursors right. If there is a selection, then it is ended. Takes a keypress."
      (lambda (keys) (command-apply kak-move-cursor-right))
      #t
      'key-sequence))

  (define kak-extend-cursor-right
    (command-create
      'kak-extend-cursor-right
      "Move the cursors right and extend the selection"
      (lambda () (when (not (text-edit-mode-is-mark-set? 0))
                   (command-apply editor-place-point-mark))
                 (command-apply editor-cursor-right #f))))

  (define kak-extend-cursor-right-keypress
    (command-create
      'kak-extend-cursor-right-keypress
      "Move the cursors right and extend the selection. Takes a keypress."
      (lambda (keys) (command-apply kak-extend-cursor-right))
      #t
      'key-sequence))

  (define kak-stop-selection
    (command-create
      'kak-stop-selection
      "Ends the selection"
      (lambda () (when (not (text-edit-mode-is-mark-set? 0))
                   (command-apply editor-remove-mark)))))

  (define kak-stop-selection-keypress
    (command-create
      'kak-stop-selection-keypress
      "Ends the selection"
      (lambda (key) (command-apply kak-stop-selection))
      #t
      'key-sequence))

  (define (kakoune-normal-mode-keymap)
    (let ((kakoune-key-map (key-map-create)))
      (key-map-insert kakoune-key-map "UP" kak-move-cursor-up-keypress)
      (key-map-insert kakoune-key-map "k"  kak-move-cursor-up-keypress)
      (key-map-insert kakoune-key-map "DOWN"  kak-move-cursor-down-keypress)
      (key-map-insert kakoune-key-map "j"  kak-move-cursor-down-keypress)
      (key-map-insert kakoune-key-map "LEFT"  kak-move-cursor-left-keypress)
      (key-map-insert kakoune-key-map "h"  kak-move-cursor-left-keypress)
      (key-map-insert kakoune-key-map "RIGHT"  kak-move-cursor-right-keypress)
      (key-map-insert kakoune-key-map "l"  kak-move-cursor-right-keypress)
      (key-map-insert kakoune-key-map "S-UP" kak-extend-cursor-up-keypress)
      (key-map-insert kakoune-key-map "K"  kak-extend-cursor-up-keypress)
      (key-map-insert kakoune-key-map "S-DOWN"  kak-extend-cursor-down-keypress)
      (key-map-insert kakoune-key-map "J"  kak-extend-cursor-down-keypress)
      (key-map-insert kakoune-key-map "S-LEFT"  kak-extend-cursor-left-keypress)
      (key-map-insert kakoune-key-map "H"  kak-extend-cursor-left-keypress)
      (key-map-insert kakoune-key-map "S-RIGHT"  kak-extend-cursor-right-keypress)
      (key-map-insert kakoune-key-map "L"  kak-extend-cursor-right-keypress)
      (key-map-insert kakoune-key-map ";" kak-stop-selection-keypress)
      (key-map-insert kakoune-key-map "i" kakoune-enter-insert-keypress)
      (key-map-insert kakoune-key-map ":" kakoune-enter-command-keypress)
      (key-map-insert kakoune-key-map "u" editor-undo-keypress)
      (key-map-insert kakoune-key-map "U" editor-redo-keypress)
      (key-map-insert kakoune-key-map "C-c" editor-cursor-add-below-keypress)
      kakoune-key-map))


  (define (kakoune-insert-mode-keymap)
    (let ((kakoune-key-map (key-map-create editor-insert-text-keypress)))
      (key-map-insert kakoune-key-map "UP" editor-cursor-up-keypress)
      (key-map-insert kakoune-key-map "DOWN" editor-cursor-down-keypress)
      (key-map-insert kakoune-key-map "LEFT" editor-cursor-left-keypress)
      (key-map-insert kakoune-key-map "RIGHT" editor-cursor-right-keypress)
      (key-map-insert kakoune-key-map "BS" editor-delete-back-keypress)
      (key-map-insert kakoune-key-map "DEL" editor-delete-forward-keypress)
      (key-map-insert kakoune-key-map "ENTER" editor-insert-newline-keypress)
      (key-map-insert kakoune-key-map "SPC" editor-insert-space-keypress)
      kakoune-key-map))

  (define (kakoune-command-mode-keymap)
    (let ((kakoune-key-map (key-map-create command-insert-text)))
      (key-map-insert kakoune-key-map "LEFT" command-cursor-left)
      (key-map-insert kakoune-key-map "RIGHT" command-cursor-right)
      (key-map-insert kakoune-key-map "BS" command-delete-back)
      (key-map-insert kakoune-key-map "DEL" command-delete-forward)
      (key-map-insert kakoune-key-map "ENTER" command-activate)
      (key-map-insert kakoune-key-map "SPC" command-insert-space)
      kakoune-key-map))

  (define (enter-normal-mode)
    (when (is-current-buffer-set?)
      (command-apply text-edit-mode-end-transaction))
    (add-key-map 'kakoune-edit (kakoune-normal-mode-keymap)))

  (define (enter-normal-mode-first-time)
    (add-key-map 'kakoune-edit (kakoune-normal-mode-keymap)))

  (define (enter-insert-mode)
    (when (is-current-buffer-set?)
      (command-apply text-edit-mode-start-transaction))
    (when (text-edit-mode-is-mark-set? 0)
      (command-apply text-edit-mode-remove-mark))
    (add-key-map 'kakoune-edit (kakoune-insert-mode-keymap)))

  (define (enter-command-mode)
    (when (is-current-buffer-set?)
      (command-apply text-edit-mode-start-transaction))
    (add-key-map 'kakoune-edit (kakoune-command-mode-keymap)))

  (define (kakoune-enter-mode kakoune-mode mode)
    (remove-key-map 'kakoune-edit)
    (cond
      ((equal? mode 'Normal) (enter-normal-mode))
      ((equal? mode 'Insert) (enter-insert-mode))
      ((equal? mode 'Command) (enter-command-mode))))


  (define (kakoune-gain-focus kakoune-mode)
    (kakoune-enter-mode kakoune-mode (kakoune-state kakoune-mode))
    (add-special-key-binding "ESC" kakoune-escape))
  (define (kakoune-lose-focus kakoune-mode)
    (remove-key-map 'kakoune-edit)
    (remove-special-key-binding "ESC"))

  (define (kakoune-mode)
    (let ((kakoune-mode (minor-mode-create 'kakoune-mode kakoune-gain-focus kakoune-lose-focus)))
      (let ((kakoune-modal (modal-create 'Normal 'kakoune-mode-change (lambda (old new) (kakoune-enter-mode kakoune-mode new)))))
        (minor-mode-data-set! kakoune-mode kakoune-modal)
        kakoune-mode)))

  (define (kakoune-config-hook buffer-name file-ext)
    (minor-mode-add buffer-name (kakoune-mode))
    (enter-normal-mode-first-time))

  (define (init-kakoune-config)
    (add-special-key-binding "C-q" editor-crash)
    (create-hook 'kakoune-mode-change)
    (add-hook 'buffer-open 'text-edit-mode text-edit-mode-file-open-hook)
    (add-hook 'buffer-open 'kakoune-mode kakoune-config-hook)))
