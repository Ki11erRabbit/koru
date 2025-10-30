(use-modules (ice-9 control))

(define async-prompt (make-prompt-tag "async"))

(define (await thing)
  (abort-to-prompt async-prompt thing))

(define (spawn future)
  (spawn-future future))

;; Recursive prompt loop
(define (run-with-prompts thunk)
  (call-with-prompt async-prompt
    thunk
    (lambda (continuation thing)
      (await-future
        (lambda (result)
          ;; After resuming, run the rest with prompts again
          (run-with-prompts (lambda () (continuation result))))
        thing)
      (values))))

(define-syntax async-do
  (syntax-rules ()
    ((async-do body ...)
      (run-with-prompts (lambda () body ...)))))

(export spawn await async-do async-prompt run-with-prompts)