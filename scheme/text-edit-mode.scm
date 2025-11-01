(import (rnrs))
(import (major-mode))
(import (koru-session))
(import (styled-text))

(define (append-line-numbers file i total-lines)
  (if (< i total-lines)
    (begin
      (styled-file-prepend file i (styled-text-create (write-line-number i total-lines #\|)))
      (append-line-numbers file (+ i 1) total-lines))
    file))

(define (modify-lines file total-lines)
  (append-line-numbers file 0 total-lines))

(define text-edit-mode (major-mode-create
                         "TextEdit"
                         modify-lines
                         #t))

(define (file-open-hook buffer-name file-ext)
  (major-mode-set! buffer-name text-edit-mode))

(add-hook "file-open" "text-edit-mode" file-open-hook)