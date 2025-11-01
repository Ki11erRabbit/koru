(import (rnrs))
(import (koru-command))

(define session-id #f)

(define (session-id-set! value)
  (set! session-id value))

(define (get-session-id)
  session-id)

(define current-major-mode #f)

(define (current-major-mode-set! value)
  (set! current-major-mode value))

(define current-buffer #f)

(define (current-buffer-set! value)
  (set! current-buffer value))

(define (get-current-buffer)
  current-buffer)

(define file-open-hook #f)

(define (file-open-hook-set! value)
  (set! file-open-hook value))
