(use-modules (async))
(use-modules (ice-9 control))



(async-do
  (let ((task1 (spawn (async-function)))
         (task2 (spawn (async-function)))
         (task3 (spawn (async-function))))
    (let ((r1 (await task1))
           (r2 (await task2))
           (r3 (await task3)))
      (validate-input (+ r1 r2 r3)))))