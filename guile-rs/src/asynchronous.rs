use std::collections::{HashMap, VecDeque};
use std::panic::AssertUnwindSafe;
use std::sync::{LazyLock, Mutex};
use futures::future::BoxFuture;
use crate::scheme_object::{SchemeObject, SchemeProcedure, SchemeString};
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

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct TaskId(usize);

struct Tasks {
    continuations: HashMap<TaskId, SchemeProcedure>,
    next_task_id: TaskId,
    free_list: VecDeque<TaskId>,
}

impl Tasks {
    pub fn new() -> Self {
        Tasks {
            continuations: HashMap::new(),
            next_task_id: TaskId(0),
            free_list: VecDeque::new(),
        }
    }

    fn insert_internal(&mut self, continuation: SchemeProcedure) -> TaskId {
        let task_id = if let Some(task_id) = self.free_list.pop_front() {
            task_id
        } else {
            let task_id = self.next_task_id;
            self.next_task_id.0 += 1;
            task_id
        };
        self.continuations.insert(task_id, continuation);
        task_id
    }

    fn remove_internal(&mut self, task_id: TaskId) -> Option<SchemeProcedure> {
        self.continuations.remove(&task_id)
    }

    pub fn insert(continuation: SchemeProcedure) -> TaskId {
        let Ok(mut tasks) = TASKS.lock() else {
            panic!("Tasks mutex poisoned");
        };
        let task_id = tasks.insert_internal(continuation);
        task_id
    }

    pub fn remove(task_id: TaskId) -> Option<SchemeProcedure> {
        let Ok(mut tasks) = TASKS.lock() else {
            panic!("Tasks mutex poisoned");
        };
        tasks.remove_internal(task_id)
    }
}

static TASKS: LazyLock<Mutex<Tasks>> = LazyLock::new(|| {
    Mutex::new(Tasks::new())
});


extern "C" fn await_rust_future(continuation: SchemeValue, future_handle: SchemeValue) -> SchemeValue {
    let Some(continuation) = SchemeObject::from(continuation).cast_procedure() else {
        guile_wrong_type_arg!("await-rust-future", 1, continuation);
    };
    let Some(mut future_handle) = SchemeObject::from(future_handle).cast_smob(FUTURE_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("await-rust-future", 2, future_handle);
    };

    let task_id = Tasks::insert(continuation);
    println!("4. Inserted Task");
    let Some(future) = future_handle.try_take() else {
        println!("try_take returned none");
        Tasks::remove(task_id);
        return SchemeValue::undefined();
    };
    println!("5. Got future");
    
    //let handle = tokio::runtime::Handle::current();

    tokio::spawn(async move {
        let result = std::panic::catch_unwind(AssertUnwindSafe(async move || {
            future.take().await
        }));

        match result {
            Ok(value) => {
                let continuation = Tasks::remove(task_id).expect("Continuation not found");
                let value = value.await;
                Guile::init(move || {
                    println!("calling continuation");
                    continuation.call1(value);
                    println!("continuation done");
                });
            }
            Err(panic) => {
                Tasks::remove(task_id).expect("Continuation not found");
                let error = format!("{:?}", panic);
                let error = SchemeString::new(error);
                
                Guile::init(move || {
                    guile_misc_error!("await-rust-future", "an error occured while completing the future", error);
                })
            }
        }
    });

    println!("6. Spawned Task");


    println!("7. about to return");
    SchemeValue::undefined()
}

pub fn async_module() {
    Guile::define_fn("await-rust-future", 2, 0, false,
        await_rust_future as extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue
    );
    let mut module: Module<()> = Module::new_default("async");
    module.add_export("await-rust-future");
    module.export();
    module.define_default();
}

#[cfg(test)]
mod tests {
    use crate::scheme_object::SchemeSmob;
    use super::*;
    
    extern "C" fn async_function() -> SchemeValue {
        let smob = FUTURE_SMOB_TAG.make(BoxFutureWrapper::new(async move {
            SchemeValue::from(10)
        }));
        
        <SchemeSmob<BoxFutureWrapper<'_>> as Into<SchemeObject>>::into(smob).into()
    }
    
    #[test]
    fn test_async_code() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        
        runtime.block_on(async {
            Guile::init(|| {
                async_module();
                Guile::define_fn("async-function", 0, 0, false,
                                 async_function as extern "C" fn() -> SchemeValue
                );
                Guile::load("scheme_test_files/async.scm").unwrap();
            });
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        });
    }
}