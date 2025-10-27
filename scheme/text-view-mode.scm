(use-modules (koru-session major-mode styled-text format))

(define (modify-line mode file total-lines)
  (let ([digit-len (length (string->list (number->string total-lines)))])
  (begin
    (cond
      ((major-mode-data mode)
        (do ([i 0]) (< i total-lines)
                    (set! file
                      (styled-file-prepend file i
                        (styled-text-create
                          (format #f "~vd|" digit-len (+ i 1))))))))
    file)))

(define mode (major-mode-create "TextView" modify-line #t))

(define (file-open-hook file-name file-ext)
  (set-major-mode file-name mode))


(add-hook "file-open-hook" "TextView" file-open-hook)