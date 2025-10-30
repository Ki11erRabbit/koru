use std::collections::{HashMap, VecDeque};
use std::panic::AssertUnwindSafe;
use std::sync::{LazyLock, Mutex};
use futures::future::BoxFuture;
use crate::scheme_object::{SchemeObject, SchemeProcedure, SchemeSmob, SchemeString};
use crate::{guile_misc_error, guile_wrong_type_arg, Guile, Module, SchemeValue, SmobData, SmobTag};


pub static FUTURE_SMOB_TAG: LazyLock<SmobTag<BoxFutureWrapper>> = LazyLock::new(|| {
    SmobTag::register("FutureScheme")
});

pub struct BoxFutureWrapper<'a> {
    future: BoxFuture<'a, SchemeValue>
}

impl<'a> BoxFutureWrapper<'a> {
    
    pub fn new(future: impl Future<Output = SchemeValue> + Send + 'static) -> Self {
        Self { future: Box::pin(future) }
    }
    fn take(self) -> BoxFuture<'a, SchemeValue> {
        self.future
    }
}

impl<'a> From<BoxFuture<'a, SchemeValue>> for BoxFutureWrapper<'a> {
    fn from(future: BoxFuture<'a, SchemeValue>) -> Self {
        Self { future }
    }
}

impl<'a> Clone for BoxFutureWrapper<'a> {
    fn clone(&self) -> Self {
        BoxFutureWrapper {
            future: Box::pin(async {
                SchemeValue::from('c')
            })
        }
    }
}

impl<'a> SmobData for BoxFutureWrapper<'a> {
    fn print(&self) -> String {
        String::from("#<Future>")
    }

    fn heap_size(&self) -> usize {
        0
    }

    fn size() -> usize {
        0
    }
}

pub static TASK_ID_SMOB_TAG: LazyLock<SmobTag<TaskId>> = LazyLock::new(|| {
    SmobTag::register("TaskId")
});

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct TaskId(usize);

impl SmobData for TaskId {
    fn print(&self) -> String {
        format!("#<task#{}>", self.0)
    }

    fn heap_size(&self) -> usize {
        0
    }
}

struct Tasks {
    continuations: HashMap<TaskId, SchemeProcedure>,
    results: HashMap<TaskId, SchemeValue>,
    next_task_id: TaskId,
    free_list: VecDeque<TaskId>,
}

impl Tasks {
    pub fn new() -> Self {
        Tasks {
            continuations: HashMap::new(),
            results: HashMap::new(),
            next_task_id: TaskId(0),
            free_list: VecDeque::new(),
        }
    }

    fn get_next_task_id(&mut self) -> TaskId {
        let task_id = if let Some(task_id) = self.free_list.pop_front() {
            task_id
        } else {
            let task_id = self.next_task_id;
            self.next_task_id.0 += 1;
            task_id
        };
        task_id
    }

    fn insert_continuation_internal(&mut self, continuation: SchemeProcedure) -> TaskId {
        let task_id = self.get_next_task_id();
        self.continuations.insert(task_id, continuation);
        task_id
    }

    fn remove_continuation_internal(&mut self, task_id: TaskId) -> Option<SchemeProcedure> {
        self.continuations.remove(&task_id)
    }

    fn insert_result_internal(&mut self, task_id: TaskId, result: SchemeValue) -> TaskId {
        self.results.insert(task_id, result);
        task_id
    }

    fn insert_task_internal(&mut self, task_id: TaskId, result: SchemeProcedure) -> TaskId {
        self.continuations.insert(task_id, result);
        task_id
    }

    fn remove_result_internal(&mut self, task_id: TaskId) -> Option<SchemeValue> {
        self.results.remove(&task_id)
    }

    pub fn insert_continuation(continuation: SchemeProcedure) -> TaskId {
        let Ok(mut tasks) = TASKS.lock() else {
            panic!("Tasks mutex poisoned");
        };
        let task_id = tasks.insert_continuation_internal(continuation);
        task_id
    }

    pub fn remove_continuation(task_id: TaskId) -> Option<SchemeProcedure> {
        let Ok(mut tasks) = TASKS.lock() else {
            panic!("Tasks mutex poisoned");
        };
        tasks.remove_continuation_internal(task_id)
    }

    pub fn insert_result(task_id: TaskId, result: SchemeValue) -> TaskId {
        let Ok(mut tasks) = TASKS.lock() else {
            panic!("Tasks mutex poisoned");
        };
        let task_id = tasks.insert_result_internal(task_id, result);
        task_id
    }

    pub fn insert_task(task_id: TaskId, continuation: SchemeProcedure) -> TaskId {
        let Ok(mut tasks) = TASKS.lock() else {
            panic!("Tasks mutex poisoned");
        };
        let task_id = tasks.insert_task_internal(task_id, continuation);
        task_id
    }

    pub fn remove_result(task_id: TaskId) -> Option<SchemeValue> {
        let Ok(mut tasks) = TASKS.lock() else {
            panic!("Tasks mutex poisoned");
        };
        tasks.remove_result_internal(task_id)
    }

    pub fn create_task_id() -> TaskId {
        let Ok(mut tasks) = TASKS.lock() else {
            panic!("Tasks mutex poisoned");
        };
        tasks.get_next_task_id()
    }
}

static TASKS: LazyLock<Mutex<Tasks>> = LazyLock::new(|| {
    Mutex::new(Tasks::new())
});


extern "C" fn await_future(continuation: SchemeValue, future_handle: SchemeValue) -> SchemeValue {
    let Some(continuation) = SchemeObject::from(continuation).cast_procedure() else {
        guile_wrong_type_arg!("await-rust-future", 1, continuation);
    };

    if let Some(mut future_handle) = SchemeObject::from(future_handle).cast_smob(FUTURE_SMOB_TAG.clone()) {
        let task_id = Tasks::insert_continuation(continuation);
        let Some(future) = future_handle.try_take() else {
            Tasks::remove_continuation(task_id);
            return SchemeValue::undefined();
        };

        tokio::spawn(async move {
            let result = std::panic::catch_unwind(AssertUnwindSafe(async move || {
                future.take().await
            }));

            match result {
                Ok(value) => {
                    let continuation = Tasks::remove_continuation(task_id).expect("Continuation not found");
                    let value = value.await;
                    Guile::init(move || {
                        continuation.call1(value);
                        SchemeObject::undefined()
                    });
                }
                Err(panic) => {
                    Tasks::remove_continuation(task_id).expect("Continuation not found");
                    let error = format!("{:?}", panic);
                    let error = SchemeString::new(error);

                    Guile::init(move || {
                        guile_misc_error!("await-rust-future", "an error occured while completing the future", error);
                    });
                }
            }
        });
        SchemeValue::undefined()
    } else if let Some(task_handle) = SchemeObject::from(future_handle).cast_smob(TASK_ID_SMOB_TAG.clone()) {
        let task_id = task_handle.borrow().clone();
        if let Some(result) = Tasks::remove_result(task_id) {
            /*let resume_fn = SchemeProcedure::new("resume-continuation-with-prompt");
            resume_fn.call2(continuation, result).into()*/
            continuation.call1(result).into()
        } else {
            Tasks::insert_task(task_id, continuation);
            SchemeValue::undefined()
        }
    } else {
        guile_wrong_type_arg!("await-rust-future", 2, future_handle);
    }
}

extern "C" fn spawn_future(future_handle: SchemeValue) -> SchemeValue {
    let Some(mut future_handle) = SchemeObject::from(future_handle).cast_smob(FUTURE_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("await-rust-future", 2, future_handle);
    };

    let task_id = Tasks::create_task_id();

    let Some(future) = future_handle.try_take() else {
        return SchemeValue::undefined();
    };

    tokio::spawn(async move {
        let result = std::panic::catch_unwind(AssertUnwindSafe(async move || {
            future.take().await
        }));
        match result {
            Ok(value) => {
                let value = value.await;
                if let Some(continuation) = Tasks::remove_continuation(task_id) {
                    Guile::init(move || {
                        continuation.call1(value);
                        SchemeObject::undefined()
                    });
                } else {
                    Tasks::insert_result(task_id, value);
                }
            }
            Err(panic) => {
                Tasks::remove_continuation(task_id).expect("Continuation not found");
                let error = format!("{:?}", panic);
                let error = SchemeString::new(error);
                Guile::init(move || {
                    guile_misc_error!("spawn-future", "an error occured while completing the future", error);
                });
            }
        }
    });

    let smob = TASK_ID_SMOB_TAG.make(task_id);
    <SchemeSmob<TaskId> as Into<SchemeObject>>::into(smob).into()
}

pub fn async_module() {
    let mut module: Module<()> = Module::new_default("async");
    module.define_default();
    
    Guile::define_fn("await-future", 2, 0, false,
        await_future as extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("spawn-future", 1, 0, false,
        spawn_future as extern "C" fn(SchemeValue) -> SchemeValue
    );
    Guile::eval(include_str!("async-support.scm"));
    module.add_export("await-future");
    module.add_export("spawn-future");
    module.add_export("spawn");
    module.add_export("await");
    module.add_export("async-do");

    module.export();
}

#[cfg(test)]
mod tests {
    use crate::scheme_object::SchemeSmob;
    use super::*;
    
    #[test]
    fn test_async01() {
        extern "C" fn async_function() -> SchemeValue {
            let smob = FUTURE_SMOB_TAG.make(BoxFutureWrapper::new(async move {
                SchemeValue::from(10)
            }));

            <SchemeSmob<BoxFutureWrapper<'_>> as Into<SchemeObject>>::into(smob).into()
        }
        extern "C" fn validate_input(value: SchemeValue) -> SchemeValue {
            let Some(number) = SchemeObject::from(value).cast_number() else {
                panic!("Expected a number");
            };
            let value = number.as_u64();
            
            assert_eq!(value, 30, "Expected value to be 30");
            
            SchemeValue::undefined()
        }
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            Guile::init(|| {
                Guile::define_fn("async-function", 0, 0, false,
                                 async_function as extern "C" fn() -> SchemeValue
                );
                Guile::define_fn("validate-input", 1, 0, false,
                                 validate_input as extern "C" fn(SchemeValue) -> SchemeValue
                );
                Guile::load("scheme_test_files/async01.scm").unwrap();
                SchemeObject::from(SchemeValue::undefined())
            });
        });
    }
}