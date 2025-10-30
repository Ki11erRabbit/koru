(use-modules (async))

(define (await rust-future)
  "Suspend current computation and wait for rust-future to complete"
  (call/cc
    (lambda (continuation)
      ;; This never returns - Rust will resume us later
      (await-rust-future continuation rust-future))))

(define-syntax async
  (syntax-rules ()
    ((async body ...)
      (lambda ()
        body ...))))

(define-syntax async-lambda
  (syntax-rules ()
    ((async-lambda args body ...)
      (lambda args
        body ...))))


(display (await (async-function)))

(sleep 1)