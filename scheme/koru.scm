(library (scheme koru)
  (export for)
  (import (rnrs))

  (define-syntax for
    (syntax-rules (from to step)
      ((for var from start to end step increment body ...)
        (let loop ((var start))
          (when (<= var end)
            body ...
            (loop (+ var increment)))))
      ((for var from start to end body ...)
        (let loop ((var start))
          (when (<= var end)
            body ...
            (loop (+ var 1)))))))
    )
