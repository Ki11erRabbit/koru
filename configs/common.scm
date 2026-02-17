(library (configs common)
  (export
    editor-cursor-up
    editor-cursor-up-keypress
    editor-cursor-down
    editor-cursor-down-keypress
    editor-cursor-left
    editor-cursor-left-keypress
    editor-cursor-left-wrap
    editor-cursor-left-wrap-keypress
    editor-cursor-right
    editor-cursor-right-keypress
    editor-cursor-right-wrap
    editor-cursor-right-wrap-keypress
    editor-cursor-line-start
    editor-cursor-line-start-keypress
    editor-cursor-line-end
    editor-cursor-line-end-keypress
    editor-cursor-buffer-start
    editor-cursor-buffer-start-keypress
    editor-cursor-buffer-end
    editor-cursor-buffer-end-keypress
    editor-cursor-add-above
    editor-cursor-add-above-keypress
    editor-cursor-add-below
    editor-cursor-add-below-keypress
    editor-remove-additional-cursors
    editor-remove-additional-cursors-keypress
    editor-insert-text
    editor-insert-text-keypress
    editor-insert-space
    editor-insert-space-keypress
    editor-insert-newline
    editor-insert-newline-keypress
    editor-delete-back
    editor-delete-back-keypress
    editor-delete-forward
    editor-delete-forward-keypress
    editor-delete-region
    editor-delete-region-keypress
    editor-place-point-mark
    editor-place-point-mark-keypress
    editor-remove-mark
    editor-remove-mark-keypress
    editor-undo
    editor-undo-keypress
    editor-redo
    editor-redo-keypress
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
    mode-state-data-change
    editor-quit)
  (import (rnrs)
    (koru-command)
    (koru-session)
    (koru-key)
    (minor-mode)
    (koru-buffer)
    (scheme koru)
    (scheme text-edit-mode))


  (define editor-cursor-up
    (command-create
      'editor-cursor-up
      "Moves the primary cursor up"
      (lambda () (let ((cursor-count (text-edit-mode-cursor-count)))
                       (for i from 0 to (- cursor-count 1)
                         (command-apply text-edit-mode-cursor-up i))))))

  (define editor-cursor-up-keypress
    (command-create
      'editor-cursor-up-keypress
      "Moves the primary cursor up in response to a keypress"
      (lambda (keys) (command-apply editor-cursor-up))
      #t
      'key-sequence))

  (define editor-cursor-down
    (command-create
      'editor-cursor-down
      "Moves the primary cursor down"
      (lambda () (let ((cursor-count (text-edit-mode-cursor-count)))
                        (for i from 0 to (- cursor-count 1)
                          (command-apply text-edit-mode-cursor-down i))))))

  (define editor-cursor-down-keypress
    (command-create
      'editor-cursor-down-keypress
      "Moves the primary cursor down in response to a keypress"
      (lambda (keys) (command-apply editor-cursor-down))
      #t
      'key-sequence))

  (define editor-cursor-left
    (command-create
      'editor-cursor-left
      "Moves the primary cursor left. Takes in a boolean to indicate to wrap or not"
      (lambda (wrap) (let ((cursor-count (text-edit-mode-cursor-count)))
                       (for i from 0 to (- cursor-count 1)
                         (command-apply text-edit-mode-cursor-left i wrap))))
      'boolean))

  (define editor-cursor-left-keypress
    (command-create
      'editor-cursor-left-keypress
      "Moves the primary cursor left in response to a keypress"
      (lambda (keys) (command-apply editor-cursor-left #f))
      #t
      'key-sequence))

  (define editor-cursor-left-wrap-keypress
    (command-create
      'editor-cursor-left-wrap-keypress
      "Moves the primary cursor left in response to a keypress. The cursor wraps if at the start of the line"
      (lambda (keys) (command-apply editor-cursor-left #t))
      #t
      'key-sequence))

  (define editor-cursor-right
    (command-create
      'editor-cursor-right
      "Moves the cursors to the right. Takes in a boolean to indicate to wrap or not"
      (lambda (wrap) (let ((cursor-count (text-edit-mode-cursor-count)))
                       (for i from 0 to (- cursor-count 1)
                         (command-apply text-edit-mode-cursor-right i wrap))))
      'boolean))

  (define editor-cursor-right-keypress
    (command-create
      'editor-cursor-right-keypress
      "Moves the primary cursor right in response to a keypress"
      (lambda (keys) (command-apply editor-cursor-right #f))
      #t
      'key-sequence))

  (define editor-cursor-right-wrap-keypress
    (command-create
      'editor-cursor-right-wrap-keypress
      "Moves the primary cursor right in response to a keypress. The cursor wraps if at the end of the line"
      (lambda (keys) (command-apply editor-cursor-right #t))
      #t
      'key-sequence))

  (define editor-cursor-line-start
    (command-create
      'editor-cursor-line-start
      "Moves the cursors to the start of a line"
      (lambda () (let ((cursor-count (text-edit-mode-cursor-count)))
                       (for i from 0 to (- cursor-count 1)
                         (command-apply text-edit-mode-cursor-line-start i))))))

  (define editor-cursor-line-start-keypress
    (command-create
      'editor-cursor-line-start-keypress
      "Moves the cursors to the start of a line in response to a keypress"
      (lambda (keys) (command-apply editor-cursor-line-start))
      #t
      'key-sequence))

  (define editor-cursor-line-end
    (command-create
      'editor-cursor-line-end
      "Moves the cursors to the end of a line"
      (lambda () (let ((cursor-count (text-edit-mode-cursor-count)))
                   (for i from 0 to (- cursor-count 1)
                     (command-apply text-edit-mode-cursor-line-end i))))))

  (define editor-cursor-line-end-keypress
    (command-create
      'editor-cursor-line-end-keypress
      "Moves the cursors to the start of a line in response to a keypress"
      (lambda (keys) (command-apply editor-cursor-line-end))
      #t
      'key-sequence))

  (define editor-cursor-buffer-start
    (command-create
      'editor-cursor-buffer-start
      "Moves the cursors to the start of the buffer"
      (lambda () (let ((cursor-count (text-edit-mode-cursor-count)))
                   (for i from 0 to (- cursor-count 1)
                     (command-apply text-edit-mode-cursor-buffer-start i))))))

  (define editor-cursor-buffer-start-keypress
    (command-create
      'editor-cursor-buffer-start-keypress
      "Moves the cursors to the start of the buffer in response to a keypress"
      (lambda (keys) (command-apply editor-cursor-buffer-start))
      #t
      'key-sequence))

  (define editor-cursor-buffer-end
    (command-create
      'editor-cursor-buffer-end
      "Moves the cursors to the end of the buffer"
      (lambda () (let ((cursor-count (text-edit-mode-cursor-count)))
                   (for i from 0 to (- cursor-count 1)
                     (command-apply text-edit-mode-cursor-buffer-end i))))))

  (define editor-cursor-buffer-end-keypress
    (command-create
      'editor-cursor-libufferne-end-keypress
      "Moves the cursors to the end of the buffer in response to a keypress"
      (lambda (keys) (command-apply editor-cursor-buffer-end))
      #t
      'key-sequence))

  (define editor-cursor-add-above
    (command-create
      'editor-cursor-add-above
      "Adds a cursor above the first cursor as long as the cursor isn't on the first line"
      (lambda () (let ((point (text-edit-mode-cursor-position 0)))
                       (when (not (= (car point) 0))
                               (command-apply text-edit-mode-cursor-create (- (car point) 1) (cdr point)))))))

  (define editor-cursor-add-above-keypress
    (command-create
      'editor-cursor-add-above-keypress
      "Adds a cursor above the first cursor as long as the cursor isn't on the first line in response to a keypress"
      (lambda (keys) (command-apply editor-cursor-add-above))
      #t
      'key-sequence))

  (define editor-cursor-add-below
    (command-create
      'editor-cursor-add-below
      "Adds a cursor below the last cursor"
      (lambda () (let ((last-cursor (- (text-edit-mode-cursor-count) 1)))
                       (let ((point (text-edit-mode-cursor-position last-cursor)))
                         (command-apply text-edit-mode-cursor-create (+ (car point) 1) (cdr point)))))))

  (define editor-cursor-add-below-keypress
    (command-create
      'editor-cursor-add-below-keypress
      "Adds a cursor below the last cursor"
      (lambda (keys) (command-apply editor-cursor-add-below))
      #t
      'key-sequence))

  (define editor-remove-additional-cursors
    (command-create
      'editor-remove-additional-cursors
      "Removes all cursors except the main cursor"
      (lambda () (let ((last-cursor (- (text-edit-mode-cursor-count) 1))
                        (main-cursor (text-edit-mode-main-cursor-index)))
                   (for i from 0 to last-cursor step -1
                     (when (not (= i main-cursor))
                       (command-apply text-edit-mode-cursor-destroy i)))))))

  (define editor-remove-additional-cursors-keypress
    (command-create
      'editor-remove-additional-cursors-keypress
      "Removes all cursors except the main cursor in response to a keypress"
      (lambda (keys) (command-apply editor-remove-additional-cursors))
      #t
      'key-sequence))

  (define editor-insert-text-keypress
    (command-create
      'editor-insert-text-keypress
      "Inserts text at all cursors from a key sequence"
      (lambda (keys) (let ((cursor-count (text-edit-mode-cursor-count)) (result (list)))
                       (for i from 0 to (- cursor-count 1)
                         (set! result (command-apply text-edit-mode-insert-key i keys)))
                       result))
      #t
      'key-sequence))

  (define editor-insert-text
    (command-create
      'editor-insert-text
      "Inserts text at the cursors from text"
      (lambda (text) (let ((cursor-count (text-edit-mode-cursor-count)))
                       (for i from 0 to (- cursor-count 1)
                         (command-apply text-edit-mode-insert-at-cursor i text))))
      'text))

  (define editor-insert-space-keypress
    (command-create
      'editor-insert-space-keypress
      "Inserts a space at each cursor in response to a keypress"
      (lambda (keys) (command-apply editor-insert-text " "))
      #t
      'key-sequence))

  (define editor-insert-newline-keypress
    (command-create
      'editor-insert-newline-keypress
      "Inserts a space at each cursor in response to a keypress"
      (lambda (keys) (command-apply editor-insert-text "\n"))
      #t
      'key-sequence))

  (define editor-delete-back
    (command-create
      'editor-delete-back
      "Deletes text before each cursor"
      (lambda () (let ((cursor-count (text-edit-mode-cursor-count)))
                       (for i from 0 to (- cursor-count 1)
                         (command-apply text-edit-mode-delete-before-cursor i))))))

  (define editor-delete-back-keypress
    (command-create
      'editor-delete-back
      "Deletes text before each cursor in response to a keypress"
      (lambda (keys) (command-apply editor-delete-back))
      #t
      'key-sequence))

  (define editor-delete-forward
    (command-create
      'editor-delete-forward
      "Deletes text at each cursor"
      (lambda () (let ((cursor-count (text-edit-mode-cursor-count)))
                       (for i from 0 to (- cursor-count 1)
                         (command-apply text-edit-mode-delete-after-cursor i))))))

  (define editor-delete-forward-keypress
    (command-create
      'editor-delete-forward-keypress
      "Deletes text at each cursor in response to a keypress"
      (lambda (keys) (command-apply editor-delete-forward))
      #t
      'key-sequence))

  (define editor-delete-region
    (command-create
      'editor-delete-region
      "Deletes text in text region of each cursor"
      (lambda () (let ((cursor-count (text-edit-mode-cursor-count)))
                       (for i from 0 to (- cursor-count 1)
                         (command-apply text-edit-mode-delete-cursor-region i))))))

  (define editor-delete-region-keypress
    (command-create
      'editor-delete-region-keypress
      "Deletes text in text region of each cursor in response to a keypress"
      (lambda (keys) (let ((cursor-count (text-edit-mode-cursor-count)))
                       (for i from 0 to (- cursor-count 1)
                         (command-apply text-edit-mode-delete-cursor-region i))))
      #t
      'key-sequence))

  (define editor-place-point-mark
    (command-create
      'editor-place-point-mark
      "Places a point mark at each cursor"
      (lambda () (let ((cursor-count (text-edit-mode-cursor-count)))
                   (for i from 0 to (- cursor-count 1)
                     (command-apply text-edit-mode-place-point-mark i))))))

  (define editor-place-point-mark-keypress
    (command-create
      'editor-place-point-mark-keypress
      "Places a point mark at each cursor in response to a keypress"
      (lambda (keys) (command-apply editor-place-point-mark))
      #t
      'key-sequence))

  (define editor-remove-mark
    (command-create
      'editor-remove-mark
      "Removes the mark at each cursor"
      (lambda () (let ((cursor-count (text-edit-mode-cursor-count)))
                   (for i from 0 to (- cursor-count 1)
                     (command-apply text-edit-mode-remove-mark i))))))

  (define editor-remove-mark-keypress
    (command-create
      'editor-remove-mark-keypress
      "Removes the mark at each cursor in response to a keypress"
      (lambda (keys) (command-apply editor-remove-mark))
      #t
      'key-sequence))


  (define editor-undo-keypress
    (command-create
      'editor-undo-keypress
      "Undoes a text modification in response to a keypress"
      (lambda (keys) (command-apply text-edit-mode-undo))
      #t
      'key-sequence))

  (define editor-redo-keypress
    (command-create
      'editor-redo
      "Redoes a text modification in response to a keypress"
      (lambda (keys) (command-apply text-edit-mode-redo))
      #t
      'key-sequence))

  (define editor-quit
    (command-create
      'editor-quit
      "Checks if this is the last open session and prompts the user if there are any unsaved files to save them. Otherwise just exits the session"
      (lambda () (session-quit)))))