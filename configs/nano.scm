(library (configs nano)
  (export init-nano-config)
  (import (rnrs)
    (koru-command)
    (koru-session)
    (koru-key)
    (minor-mode)
    (koru-buffer)
    (configs common)
    (scheme text-edit-mode))

  (define (nano-state-default)
    '("edit" "" '()))

  (define (nano-state-name state)
    (car state))
  (define (nano-state-name-change state new)
    (cons new (cdr state)))

  (define (nano-state-command-prefix state)
    (car (cdr state)))
  (define (nano-state-command-prefix-change state new)
    (cons (car state) (cons new (cdr (cdr state)))))

  (define (nano-state-command-callback state)
    (car (cdr (cdr state))))
  (define (nano-state-command-callback-change state new)
    (cons (car state) (cons (car (cdr state)) (cons new (cdr (cdr (cdr state)))))))

  (define (nano-state nano-mode)
    (nano-state-name (minor-mode-data nano-mode)))
  (define (nano-state-set! nano-mode new)
    (minor-mode-data-set! nano-mode (nano-state-name-change (minor-mode-data nano-mode) new)))

  (define (nano-prefix nano-mode)
    (nano-state-command-prefix (minor-mode-data nano-mode)))
  (define (nano-prefix-set! nano-mode new)
    (minor-mode-data-set! nano-mode (nano-state-command-prefix-change (minor-mode-data nano-mode) new)))

  (define (nano-callback nano-mode)
    (nano-state-command-callback (minor-mode-data nano-mode)))
  (define (nano-callback-set! nano-mode new)
    (minor-mode-data-set! nano-mode (nano-state-command-callback-change (minor-mode-data nano-mode) new)))

  (define (nano-mode)
    (let ((nano-mode (minor-mode-create "nano-mode" nano-gain-focus nano-lose-focus (nano-state-default))))
      nano-mode))

  (define (nano-gain-focus nano-mode)
    (nano-change-state-internal nano-mode (nano-state nano-mode)))

  (define (nano-lose-focus nano-mode)
    (remove-key-map "nano-edit"))

  (define (nano-config-hook buffer-name file-ext)
    (minor-mode-add buffer-name (nano-mode)))

  (define (init-nano-config)
    (create-hook "nano-mode-change")
    (add-hook "buffer-open" "text-edit-mode" text-edit-mode-file-open-hook)
    (add-hook "buffer-open" "nano-mode" nano-config-hook))




  (define (nano-write-callback)
    (let ((file-name (command-bar-take)))
      (buffer-save-as (current-buffer-name) file-name)))

  (define (nano-change-state-internal nano-mode state)
    (cond
      ((equal? state "edit") (change-keymap nano-edit-key-map))
      ((equal? state "read") (begin
                               (nano-callback-set! nano-mode
                                 (lambda ()
                                   (let ((file-name (command-bar-take)))
                                     (display "TODO: read file and dump it into the current focused buffer at the cursor\n")
                                     (display file-name)
                                     (display "\n"))))
                               (change-keymap nano-read-key-map)))
      ((equal? state "search") '())
      ((equal? state "write") (begin
                                (nano-callback-set! nano-mode nano-write-callback)
                                (change-keymap nano-write-key-map)))
      ((equal? state "exit") (begin
                               (nano-callback-set! nano-mode
                                 (lambda ()
                                   '()))
                               (change-keymap nano-exit-key-map)))
      ((equal? state "exit-write") (begin
                                     (nano-callback-set! nano-mode
                                       (lambda ()
                                         (nano-write-callback)
                                         (display "TODO: exit the editor after saving")))
                                     (change-keymap nano-write-key-map))))
    (emit-hook "nano-mode-change" (nano-state nano-mode) state)
    (nano-state-set! nano-mode state))

  (define (nano-change-state nano-mode state)
    (if (not (equal? (nano-state nano-mode) state))
      (nano-change-state-internal nano-mode state)
      '()))

  (define (change-keymap keymap-fn)
    (remove-key-map "nano-edit")
    (add-key-map "nano-edit" (keymap-fn)))

  (define nano-eat-key-press
    (command-create
      "nano-eat-key-press"
      "Takes in any key sequence and reports that it has been used"
      (lambda (keys) #t)
      "key-sequence"))

  (define nano-edit-mode
    (command-create
      "nano-edit-mode"
      "Puts editor state back into default editing state"
      (lambda (keys)
        (let ((nano-mode (minor-mode-get "nano-mode")))
          (nano-prefix-set! nano-mode "")
          (command-bar-take)
          (command-bar-update)
          (command-bar-hide)
          (nano-change-state nano-mode "edit")))
      "key-sequence"))

  (define nano-write-mode
    (command-create
      "nano-write-mode"
      "Prompts the user to enter a file name to write to"
      (lambda (keys)
        (let ((nano-mode (minor-mode-get "nano-mode")))
          (nano-prefix-set! nano-mode "File Name to write : ")
          (command-bar-insert (current-buffer-name))
          (command-bar-show)
          (command-bar-update (nano-prefix nano-mode))
          (nano-change-state nano-mode "write")))
      "key-sequence"))

  (define nano-read-mode
    (command-create
      "nano-read-mode"
      "Prompts the user to enter a file name to read from to insert into the buffer"
      (lambda (keys)
        (let ((nano-mode (minor-mode-get "nano-mode")))
          (nano-prefix-set! nano-mode "File to insert from home directory: ")
          (command-bar-show)
          (command-bar-update (nano-prefix nano-mode))
          (nano-change-state nano-mode "read")))
      "key-sequence"))

  (define nano-exit-mode
    (command-create
      "nano-exit-mode"
      "Exits the editor if there are no changes in the buffer. Otherwise, prompt the user to save"
      (lambda (keys)
        (let ((nano-mode (minor-mode-get "nano-mode")))
          (if #f ;TODO: replace with an actual check against the current buffer
            (begin
              (nano-prefix-set! nano-mode "Save modified buffer (ANSWERING \"No\" WILL DESTROY CHANGES) ? ")
              (command-bar-show)
              (command-bar-update (nano-prefix nano-mode))
              (nano-change-state nano-mode "exit"))
            (display "TODO: add function that quits the session\n"))))
      "key-sequence"))

  (define nano-exit-write-mode
    (command-create
      "nano-exit-mode"
      "Prompts the user to save a file, then exits"
      (lambda (keys)
        (let ((nano-mode (minor-mode-get "nano-mode")))
          (nano-prefix-set! nano-mode "File Name to write : ")
          (command-bar-insert (current-buffer-name))
          (command-bar-show)
          (command-bar-update (nano-prefix nano-mode))
          (nano-change-state nano-mode "exit-write")))
      "key-sequence"))

  (define command-insert-text
    (command-create
      "command-insert-text"
      "Inserts text into the command bar from a key sequence"
      (lambda (keys)
        (command-bar-insert-key keys)
        (command-bar-update (nano-prefix (minor-mode-get "nano-mode"))))
      "key-sequence"))

  (define command-insert-space
    (command-create
      "command-insert-text"
      "Inserts text into the command bar from a key sequence"
      (lambda (keys) (command-bar-insert " ")
                     (command-bar-update (nano-prefix (minor-mode-get "nano-mode"))))
      "key-sequence"))

  (define command-submit-return
    (command-create
      "command-activate"
      "Activates the command"
      (lambda (keys)
        (let ((nano-mode (minor-mode-get "nano-mode")))
          (let ((callback (nano-callback nano-mode)))
            (callback)
            (command-bar-update)
            (command-bar-hide)
            (nano-prefix-set! nano-mode "")
            (nano-change-state nano-mode "edit"))))
      "key-sequence"))

  (define command-delete-back
    (command-create
      "command-delete-back"
      "Deletes backwards in the command bar"
      (lambda (keys)
        (command-bar-delete-back)
        (command-bar-update (nano-prefix (minor-mode-get "nano-mode"))))
      "key-sequence"))

  (define command-delete-forward
    (command-create
      "command-delete-forward"
      "Deletes forwards in the command bar"
      (lambda (keys)
        (command-bar-delete-forward)
        (command-bar-update (nano-prefix (minor-mode-get "nano-mode"))))
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

  (define nano-crash
    (command-create
      "command-crash"
      "Crashes the editor to see logs"
      (lambda (keys) (crash))
      "key-sequence"))


  (define (nano-edit-key-map)
    (let ((nano-key-map (key-map-create editor-insert-text)))
      (key-map-insert nano-key-map "UP" editor-cursor-up)
      (key-map-insert nano-key-map "DOWN" editor-cursor-down)
      (key-map-insert nano-key-map "LEFT" editor-cursor-left)
      (key-map-insert nano-key-map "RIGHT" editor-cursor-right)
      (key-map-insert nano-key-map "BS" editor-delete-back)
      (key-map-insert nano-key-map "DEL" editor-delete-forward)
      (key-map-insert nano-key-map "ENTER" editor-return)
      (key-map-insert nano-key-map "SPC" editor-insert-space)
      (key-map-insert nano-key-map "C-x" nano-exit-mode)
      (key-map-insert nano-key-map "C-o" nano-write-mode)
      (key-map-insert nano-key-map "C-r" nano-read-mode)
      (key-map-insert nano-key-map "C-c" nano-crash)
      nano-key-map))

  (define (nano-write-key-map)
    (let ((nano-key-map (key-map-create command-insert-text)))
      (key-map-insert nano-key-map "LEFT" command-cursor-left)
      (key-map-insert nano-key-map "RIGHT" command-cursor-right)
      (key-map-insert nano-key-map "BS" command-delete-back)
      (key-map-insert nano-key-map "DEL" command-delete-forward)
      (key-map-insert nano-key-map "ENTER" command-submit-return)
      (key-map-insert nano-key-map "SPC" command-insert-space)
      (key-map-insert nano-key-map "C-c" nano-edit-mode)
      nano-key-map))

  (define (nano-exit-key-map)
    (let ((nano-key-map (key-map-create nano-eat-key-press)))
      (key-map-insert nano-key-map "y" nano-exit-write-mode)
      (key-map-insert nano-key-map "n" nano-edit-mode)
      (key-map-insert nano-key-map "C-c" nano-edit-mode)
      nano-key-map))

  (define (nano-read-key-map)
    (let ((nano-key-map (key-map-create command-insert-text)))
      (key-map-insert nano-key-map "LEFT" command-cursor-left)
      (key-map-insert nano-key-map "RIGHT" command-cursor-right)
      (key-map-insert nano-key-map "BS" command-delete-back)
      (key-map-insert nano-key-map "DEL" command-delete-forward)
      (key-map-insert nano-key-map "ENTER" command-submit-return)
      (key-map-insert nano-key-map "SPC" command-insert-space)
      (key-map-insert nano-key-map "C-c" nano-edit-mode)
      nano-key-map))


  )