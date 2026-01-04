(library (configs emacs)
  (export init-emacs-config)
  (import (rnrs)
    (koru-command)
    (koru-session)
    (koru-key)
    (minor-mode)
    (koru-buffer)
    (scheme text-edit-mode))

  ;(define-record-type emacs-data (fields (mutable state) (mutable command-prefix) (mutable command-suffix) (mutable command-callback)))

  (define (emacs-data-default)
    '("edit" "" "" '()))
  (define (emacs-data-state data)
    (car data))
  (define (emacs-data-state-set data value)
    (cons value (cdr data)))
  (define (emacs-data-command-prefix data)
    (car (cdr data)))
  (define (emacs-data-command-prefix-set data value)
      (cons (car data) (cons value (cdr (cdr data)))))
  (define (emacs-data-command-suffix data)
    (car (cdr (cdr data))))
  (define (emacs-data-command-suffix-set data value)
    (cons (car data) (cons (car (cdr data)) (cons value (cdr (cdr (cdr data)))))))
  (define (emacs-data-command-callback data)
    (car (cdr (cdr (cdr data)))))
  (define (emacs-data-command-callback-set data value)
    (cons (car data) (cons (car (cdr data)) (cons (car (cdr (cdr data))) (cons value (cdr (cdr (cdr (cdr data)))))))))

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
                       (command-bar-take)
                       (command-bar-update)
                       (command-bar-hide)
                       (command-apply text-edit-mode-remove-mark 0)
                       (emacs-change-state (minor-mode-get "emacs-mode") "edit" #f)))
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

  (define editor-insert-space
    (command-create
      "editor-insert-text"
      "Inserts text at the primary cursor"
      (lambda (keys)
        (command-apply text-edit-mode-insert-at-cursor 0 " ")
        (command-apply text-edit-mode-cursor-right 0 #f))
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

  (define emacs-enter-command
    (command-create
      "emacs-enter-command"
      "Enters into command mode"
      (lambda (keys)
        (let ((emacs-mode (minor-mode-get "emacs-mode")))
          (emacs-mode-prefix-set! emacs-mode "Enter a command: ")
          (command-bar-update "Enter a command: ")
          (emacs-mode-callback-set!
            emacs-mode
              (lambda ()
                (command-bar-take)
                (command-bar-update)
                (command-bar-hide)))
            (emacs-change-state (minor-mode-get "emacs-mode") "command" #f)))
      "key-sequence"))

  (define command-insert-text
    (command-create
      "command-insert-text"
      "Inserts text into the command bar from a key sequence"
      (lambda (keys) (command-bar-insert-key keys)
                     (command-bar-update (emacs-mode-prefix (minor-mode-get "emacs-mode"))))
      "key-sequence"))

  (define command-insert-space
    (command-create
      "command-insert-text"
      "Inserts text into the command bar from a key sequence"
      (lambda (keys) (command-bar-insert " ")
                     (command-bar-update (emacs-mode-prefix (minor-mode-get "emacs-mode"))))
      "key-sequence"))

  (define command-activate
    (command-create
      "command-activate"
      "Activates the command"
      (lambda (keys)
        (letrec ((emacs-mode (minor-mode-get "emacs-mode"))
               (callback (emacs-mode-callback emacs-mode)))
          (callback)
          (emacs-mode-prefix-set! emacs-mode "")
          (emacs-mode-suffix-set! emacs-mode "")
          (emacs-change-state (minor-mode-get "emacs-mode") "edit" #f)))
      "key-sequence"))

  (define command-delete-back
    (command-create
      "command-delete-back"
      "Deletes backwards in the command bar"
      (lambda (keys)
        (command-bar-delete-back)
        (command-bar-update (emacs-mode-prefix (minor-mode-get "emacs-mode"))))
      "key-sequence"))

  (define command-delete-forward
    (command-create
      "command-delete-forward"
      "Deletes forwards in the command bar"
      (lambda (keys)
        (command-bar-delete-forward)
        (command-bar-update (emacs-mode-prefix (minor-mode-get "emacs-mode"))))
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

  (define editor-save
    (command-create
      "editor-save"
      "Saves the currently focused buffer if it is an open file"
      (lambda (keys) (emacs-save))
      "key-sequence"))

  (define editor-save-as
    (command-create
      "editor-save-as"
      "Saves the currently focused buffer as a different or new file"
      (lambda (keys) (emacs-save-as))
      "key-sequence"))

  (define (emacs-save)
    (let ((path (buffer-get-path (current-buffer-name))))
      (cond
        ((null? path) (display "TODO: add way to send messages to frontend"))
        (#t (buffer-save (current-buffer-name))))))

  (define (emacs-save-as-callback)
    (let ((bar-text (command-bar-take)))
      (command-bar-update)
      (command-bar-hide)
      (buffer-save-as (current-buffer-name) bar-text)))

  (define (emacs-save-as)
    (let ((emacs-mode (minor-mode-get "emacs-mode")))
      (emacs-change-state emacs-mode "command" #f)
      (emacs-mode-prefix-set! emacs-mode "Enter a file path: ")
      (command-bar-update "Enter a file path: ")
      (emacs-mode-callback-set!
        emacs-mode
        emacs-save-as-callback)
      ))

  (define (emacs-editor)
    (let ((emacs-editor-key-map (key-map-create editor-insert-text)))
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
      (key-map-insert emacs-editor-key-map "SPC" editor-insert-space)
      (key-map-insert emacs-editor-key-map "C-SPC" editor-place-point-mark)
      ;(key-map-insert emacs-editor-key-map "C-g" editor-remove-mark)
      (key-map-insert emacs-editor-key-map "C-w" editor-delete-region)
      (key-map-insert emacs-editor-key-map "C-_" editor-undo)
      (key-map-insert emacs-editor-key-map "C-x u" editor-redo)
      (key-map-insert emacs-editor-key-map "A-x" emacs-enter-command)
      (key-map-insert emacs-editor-key-map "C-x C-s" editor-save)
      (key-map-insert emacs-editor-key-map "C-x C-w" editor-save-as)
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
    (add-key-map "emacs-edit" (emacs-editor)))

  (define (emacs-command-state-keymap)
    (add-key-map "emacs-edit" (emacs-command)))

  (define (emacs-change-state-internal emacs-mode state)
    (remove-key-map "emacs-edit")
    (cond
      ((equal? state "edit") (emacs-editor-state-keymap))
      ((equal? state "command") (emacs-command-state-keymap)))
    (emacs-mode-state-set! emacs-mode state))

  (define (emacs-change-state emacs-mode state override)
    (if (and (not (equal? (emacs-mode-state emacs-mode) state)) (not override))
      (emacs-change-state-internal emacs-mode state)
      '()))

  (define (emacs-config-setup emacs-mode state)
    (emacs-change-state emacs-mode state #t)
    (add-special-key-binding "C-g" editor-remove-mark))

  (define (emacs-mode-state-set! emacs-mode mode)
    (minor-mode-data-set! emacs-mode (emacs-data-state-set (minor-mode-data emacs-mode) mode)))
  (define (emacs-mode-state emacs-mode)
    (emacs-data-state (minor-mode-data emacs-mode)))

  (define (emacs-mode-prefix-set! emacs-mode prefix)
    (minor-mode-data-set! emacs-mode (emacs-data-command-prefix-set (minor-mode-data emacs-mode) prefix)))
  (define (emacs-mode-prefix emacs-mode)
    (emacs-data-command-prefix (minor-mode-data emacs-mode)))

  (define (emacs-mode-suffix-set! emacs-mode suffix)
    (minor-mode-data-set! emacs-mode (emacs-data-command-suffix-set (minor-mode-data emacs-mode) suffix)))
  (define (emacs-mode-suffix emacs-mode)
    (emacs-data-command-suffix (minor-mode-data emacs-mode)))

  (define (emacs-mode-callback-set! emacs-mode callback)
    (minor-mode-data-set! emacs-mode (emacs-data-command-callback-set (minor-mode-data emacs-mode) callback)))
  (define (emacs-mode-callback emacs-mode)
    (emacs-data-command-callback (minor-mode-data emacs-mode)))

  (define (emacs-mode-gain-focus minor-mode)
    (emacs-config-setup minor-mode (emacs-mode-callback minor-mode)))
  (define (emacs-mode-lose-focus minor-mode)
    (remove-key-map "emacs-edit")
    (remove-special-key-binding "C-g"))

  (define (emacs-edit-mode)
    (let ((emacs-mode (minor-mode-create "emacs-mode" emacs-mode-gain-focus emacs-mode-gain-focus (emacs-data-default))))
      (emacs-editor-state-keymap)
      emacs-mode))

  (define (emacs-config-hook buffer-name file-ext)
    (minor-mode-add buffer-name (emacs-edit-mode)))

  (define (init-emacs-config)
    (add-hook "buffer-open" "text-edit-mode" text-edit-mode-file-open-hook)
    (add-hook "buffer-open" "emacs-mode" emacs-config-hook)))