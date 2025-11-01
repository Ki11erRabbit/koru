(import (rnrs))
(import (major-mode))
(import (koru-session))

(define (modify-lines file total-lines)
  file)

(define text-edit-mode (major-mode-create
                         "TextEdit"
                         modify-lines
                         #t))

(define (file-open-hook buffer-name file-ext)
  (major-mode-set! buffer-name text-edit-mode))

(add-hook "file-open" "text-edit-mode" file-open-hook)