(library (configs emacs)
  (export init-emacs-config)
  (import (rnrs)
    (koru-command)
    (koru-session)
    (koru-key)
    (minor-mode)
    (koru-buffer)
    (koru-modal)
    (configs common)
    (scheme text-edit-mode))

  (define (emacs-state emacs-mode)
    (modal-state (minor-mode-data emacs-mode)))
  (define (emacs-state-set! emacs-mode new)
    (modal-state-set! (minor-mode-data emacs-mode) new))

  (define (emacs-prefix emacs-mode)
    (modal-prefix (minor-mode-data emacs-mode)))
  (define (emacs-prefix-set! emacs-mode new)
    (modal-prefix-set! (minor-mode-data emacs-mode) new))

  (define (emacs-callback-set! emacs-mode new)
    (modal-callback-set! (minor-mode-data emacs-mode) new))
  (define (emacs-callback-apply emacs-mode)
    (modal-callback-apply (minor-mode-data emacs-mode)))

  (define editor-crash
    (command-create
      'editor-crash
      "Crashes the editor"
      (lambda (keys) (crash))
      #t
      'key-sequence))

  (define emacs-cancel
    (command-create
      'emacs-cancel
      "Removes the mark at the primary cursor and flushes the keybuffer"
      (lambda () (begin
                       (flush-key-buffer)
                       (command-bar-take)
                       (command-bar-update)
                       (command-bar-hide)
                       (command-apply editor-remove-mark)
                       (emacs-state-set! (minor-mode-get 'emacs-mode) 'edit)))))

  (define emacs-cancel-keypress
    (command-create
      'emacs-cancel-keypress
      "Removes the mark at the primary cursor and flushes the keybuffer in response to a keypress"
      (lambda (keys) (command-apply emacs-cancel))
      #t
      'key-sequence))

  (define emacs-enter-command
    (command-create
      'emacs-enter-command
      "Enters into command mode"
      (lambda (keys)
        (let ((emacs-mode (minor-mode-get 'emacs-mode)))
          (emacs-prefix-set! emacs-mode "Enter a command: ")
          (command-bar-show)
          (command-bar-update "Enter a command: ")
          (emacs-callback-set!
            emacs-mode
              (lambda ()
                (command-bar-take)))
            (emacs-state-set! emacs-mode 'command)))
      'key-sequence))

  (define command-insert-text
    (command-create
      'command-insert-text
      "Inserts text into the command bar from a key sequence"
      (lambda (keys) (command-bar-insert-key keys)
                     (command-bar-update (emacs-prefix (minor-mode-get 'emacs-mode))))
      #t
      'key-sequence))

  (define command-insert-space
    (command-create
      'command-insert-text
      "Inserts text into the command bar from a key sequence"
      (lambda (keys) (command-bar-insert " ")
                     (command-bar-update (emacs-prefix (minor-mode-get 'emacs-mode))))
      #t
      'key-sequence))

  (define command-activate
    (command-create
      'command-activate
      "Activates the command"
      (lambda (keys)
        (let ((emacs-mode (minor-mode-get 'emacs-mode)))
            (emacs-callback-apply emacs-mode)
            (command-bar-update)
            (command-bar-hide)
            (emacs-prefix-set! emacs-mode "")
            (emacs-state-set! emacs-mode 'edit)))
      #t
      'key-sequence))

  (define command-delete-back
    (command-create
      'command-delete-back
      "Deletes backwards in the command bar"
      (lambda (keys)
        (command-bar-delete-back)
        (command-bar-update (emacs-prefix (minor-mode-get 'emacs-mode))))
      #t
      'key-sequence))

  (define command-delete-forward
    (command-create
      'command-delete-forward
      "Deletes forwards in the command bar"
      (lambda (keys)
        (command-bar-delete-forward)
        (command-bar-update (emacs-prefix (minor-mode-get 'emacs-mode))))
      #t
      'key-sequence))

  (define command-cursor-left
    (command-create
      'command-cursor-left
      "Moves the cursor to the left in the command bar"
      (lambda (keys)
        (command-bar-left)
        (command-bar-update (emacs-prefix (minor-mode-get 'emacs-mode))))
      #t
      'key-sequence))

  (define command-cursor-right
    (command-create
      'command-cursor-right
      "Moves the cursor to the right in the command bar"
      (lambda (keys)
        (command-bar-right)
        (command-bar-update (emacs-prefix (minor-mode-get 'emacs-mode))))
      #t
      'key-sequence))

  (define editor-save
    (command-create
      'editor-save
      "Saves the currently focused buffer if it is an open file"
      (lambda (keys) (emacs-save))
      #t
      'key-sequence))

  (define editor-save-as
    (command-create
      'editor-save-as
      "Saves the currently focused buffer as a different or new file"
      (lambda (keys) (emacs-save-as))
      #t
      'key-sequence))

  (define (emacs-save)
    (let ((path (buffer-get-path (current-buffer-name))))
      (cond
        ((null? path) (display "TODO: add way to send messages to frontend"))
        (#t (buffer-save (current-buffer-name))))))

  (define (emacs-save-as-callback)
    (let ((bar-text (command-bar-take)))
      (buffer-save-as (current-buffer-name) bar-text)))

  (define (emacs-save-as)
    (let ((emacs-mode (minor-mode-get 'emacs-mode)))
      (emacs-state-set! emacs-mode 'command)
      (emacs-prefix-set! emacs-mode "Enter a file path: ")
      (command-bar-show)
      (command-bar-update "Enter a file path: ")
      (emacs-callback-set!
        emacs-mode
        emacs-save-as-callback)))

  (define (emacs-editor)
    (let ((emacs-editor-key-map (key-map-create editor-insert-text-keypress)))
      (key-map-insert emacs-editor-key-map "UP" editor-cursor-up-keypress)
      (key-map-insert emacs-editor-key-map "DOWN" editor-cursor-down-keypress)
      (key-map-insert emacs-editor-key-map "LEFT" editor-cursor-left-keypress)
      (key-map-insert emacs-editor-key-map "RIGHT" editor-cursor-right-keypress)
      (key-map-insert emacs-editor-key-map "C-p" editor-cursor-up-keypress)
      (key-map-insert emacs-editor-key-map "C-n" editor-cursor-down-keypress)
      (key-map-insert emacs-editor-key-map "C-b" editor-cursor-left-keypress)
      (key-map-insert emacs-editor-key-map "C-f" editor-cursor-right-keypress)
      (key-map-insert emacs-editor-key-map "BS" editor-delete-back-keypress)
      (key-map-insert emacs-editor-key-map "DEL" editor-delete-forward-keypress)
      (key-map-insert emacs-editor-key-map "ENTER" editor-insert-newline-keypress)
      (key-map-insert emacs-editor-key-map "SPC" editor-insert-space-keypress)
      (key-map-insert emacs-editor-key-map "C-SPC" editor-place-point-mark-keypress)
      (key-map-insert emacs-editor-key-map "C-w" editor-delete-region-keypress)
      (key-map-insert emacs-editor-key-map "C-_" editor-undo-keypress)
      (key-map-insert emacs-editor-key-map "C-x u" editor-redo-keypress)
      (key-map-insert emacs-editor-key-map "A-x" emacs-enter-command)
      (key-map-insert emacs-editor-key-map "C-x C-s" editor-save)
      (key-map-insert emacs-editor-key-map "C-x C-w" editor-save-as)
      (key-map-insert emacs-editor-key-map "C-x C-c" editor-crash)
      emacs-editor-key-map))

  (define (emacs-command)
    (let ((emacs-editor-key-map (key-map-create command-insert-text)))
      (key-map-insert emacs-editor-key-map "LEFT" command-cursor-left)
      (key-map-insert emacs-editor-key-map "RIGHT" command-cursor-right)
      (key-map-insert emacs-editor-key-map "C-b" command-cursor-left)
      (key-map-insert emacs-editor-key-map "C-f" command-cursor-right)
      (key-map-insert emacs-editor-key-map "BS" command-delete-back)
      (key-map-insert emacs-editor-key-map "DEL" command-delete-forward)
      (key-map-insert emacs-editor-key-map "ENTER" command-activate)
      (key-map-insert emacs-editor-key-map "SPC" command-insert-space)
      emacs-editor-key-map))

  (define (emacs-editor-state-keymap)
    (add-key-map 'emacs-edit (emacs-editor)))

  (define (emacs-command-state-keymap)
    (add-key-map 'emacs-edit (emacs-command)))

  (define (emacs-change-state emacs-mode state)
    (remove-key-map 'emacs-edit)
    (cond
      ((equal? state 'edit) (emacs-editor-state-keymap))
      ((equal? state 'command) (emacs-command-state-keymap))))


  (define (emacs-config-setup emacs-mode state)
    (emacs-change-state emacs-mode state)
    (add-special-key-binding "C-g" emacs-cancel-keypress))

  (define (emacs-mode-gain-focus minor-mode)
    (emacs-config-setup minor-mode (emacs-state minor-mode)))
  (define (emacs-mode-lose-focus minor-mode)
    (remove-key-map 'emacs-edit)
    (remove-special-key-binding "C-g"))

  (define (emacs-edit-mode)
    (let ((emacs-mode (minor-mode-create 'emacs-mode emacs-mode-gain-focus emacs-mode-lose-focus)))
      (let ((modal-data (modal-create 'edit 'emacs-mode-change (lambda (old new) (emacs-change-state emacs-mode new)))))
        (minor-mode-data-set! emacs-mode modal-data)
        emacs-mode)))

  (define (emacs-config-hook buffer-name file-ext)
    (minor-mode-add buffer-name (emacs-edit-mode)))

  (define (init-emacs-config)
    (create-hook 'emacs-mode-change)
    (add-hook 'buffer-open 'text-edit-mode text-edit-mode-file-open-hook)
    (add-hook 'buffer-open 'emacs-mode emacs-config-hook)))