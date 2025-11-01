(import (rnrs))
(import (major-mode))

(define (modify-line file total-line)
  file)

(define text-edit-mode (major-mode-create
                         "TextEdit"
                         modify-line
                         #t))

(define (file-open-hook buffer-name file-ext)
  (major-mode-set! buffer-name text-edit-mode))

(add-hook "file-open", "text-edit-mode" file-open-hook)