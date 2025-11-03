(import (rnrs))
(import (major-mode))
(import (koru-session))
(import (styled-text))

(define (prepend-line current-line total-lines)
  (if (major-mode-data text-edit-mode)
    (styled-text-create (write-line-number current-line total-lines #\|))
    (modify-line-default current-line total-lines)))

(define text-edit-mode (major-mode-create
                         "TextEdit"
                         prepend-line
                         modify-line-default
                         #t))

(define (file-open-hook buffer-name file-ext)
  (major-mode-set! buffer-name text-edit-mode))

(add-hook "file-open" "text-edit-mode" file-open-hook)