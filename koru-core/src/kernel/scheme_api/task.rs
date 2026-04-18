use std::collections::VecDeque;
use std::sync::LazyLock;
use scheme_rs::exceptions::Exception;
use scheme_rs::num::Number;
use scheme_rs::proc::Procedure;
use scheme_rs::registry::bridge;
use scheme_rs::value::Value;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

static TASK_MANAGER: LazyLock<Mutex<TaskManager>> = LazyLock::new(|| {
    Mutex::new(TaskManager::new())
});

struct TaskGroup {
    task: Vec<Option<JoinHandle<Result<Vec<Value>, Exception>>>>,
    free_list: VecDeque<usize>
}

impl TaskGroup {
    fn new() -> TaskGroup {
        Self {
            task: Vec::new(),
            free_list: VecDeque::new()
        }
    }

    fn new_task_internal(&mut self, func: Procedure) -> usize {
        let id = if let Some(id) = self.free_list.pop_front() {
            id
        } else {
            self.task.len()
        };
        let handle = tokio::spawn(async move {
            func.call(&[]).await
        });
        while id < self.task.len() {
            self.task.push(None);
        }
        self.task[id] = Some(handle);

        id
    }

    fn get_task_internal(&mut self, id: usize) -> Option<JoinHandle<Result<Vec<Value>, Exception>>> {
        if self.task[id].is_none() {
            return None;
        };
        self.free_list.push_back(id);
        self.task[id].take()
    }

    fn cancel_task_internal(&mut self, id: usize) {
        if let Some(task) = self.get_task_internal(id) {
            task.abort();
        }
    }

    fn cancel_tasks_internal(&mut self) {
        for (i, task) in self.task.iter_mut().enumerate() {
            if let Some(task) = task.take() {
                task.abort();
                self.free_list.push_back(i);
            }
        }
    }
    
    fn remove_completed_tasks(&mut self) {
        for (i, mut task) in self.task.iter_mut().enumerate() {
            if let Some(the_task) = &mut task && the_task.is_finished() {
                task.take();
                self.free_list.push_back(i);
            }
        }
    }
}

pub struct TaskManager {
    tasks: TaskGroup,
    ephemeral_tasks: TaskGroup,
}

impl TaskManager {
    fn new() -> Self {
        Self {
            tasks: TaskGroup::new(),
            ephemeral_tasks: TaskGroup::new(),
        }
    }
    
    pub async fn new_task(func: Procedure) -> usize {
        let mut guard = TASK_MANAGER.lock().await;
        guard.tasks.new_task_internal(func)
    }

    pub async fn new_emphemeral_task(func: Procedure) -> usize {
        let mut guard = TASK_MANAGER.lock().await;
        let out = guard.ephemeral_tasks.new_task_internal(func);
        guard.ephemeral_tasks.remove_completed_tasks();
        out
    }
    
    pub async fn get_task(id: usize) -> Option<JoinHandle<Result<Vec<Value>, Exception>>> {
        let mut guard = TASK_MANAGER.lock().await;
        guard.tasks.get_task_internal(id)
    }
    
    pub async fn cancel_task(id: usize) {
        let mut guard = TASK_MANAGER.lock().await;
        guard.tasks.cancel_task_internal(id);
    }
    
    pub async fn cancel_all_tasks() {
        let mut guard = TASK_MANAGER.lock().await;
        guard.tasks.cancel_tasks_internal();
        guard.ephemeral_tasks.cancel_tasks_internal();
    }
}

#[bridge(name = "spawn-task", lib = "(koru-task)")]
pub async fn spawn_task(func: &Value) -> Result<Vec<Value>, Exception> {
    let func: Procedure = func.clone().try_into()?;
    let id = TaskManager::new_task(func).await;
    let number = Number::from(id);
    Ok(vec![Value::from(number)])
}

#[bridge(name = "spawn-ephemeral-task", lib = "(koru-task)")]
pub async fn spawn_ephemeral_task(func: &Value) -> Result<Vec<Value>, Exception> {
    let func: Procedure = func.clone().try_into()?;
    TaskManager::new_emphemeral_task(func).await;
    Ok(vec![])
}

#[bridge(name = "await-task", lib = "(koru-task)")]
pub async fn await_task(id: &Value) -> Result<Vec<Value>, Exception> {
    let id: Number = id.clone().try_into()?;
    let id: usize = id.try_into()?;
    let handle = TaskManager::get_task(id).await;
    let Some(handle) = handle else {
        return Err(Exception::error("No task associated with that id"))
    };
    handle.await.unwrap_or_else(|err| {
        if err.is_cancelled() {
            Ok(vec![])
        } else {
            Err(Exception::error(err))
        }
    })
}

#[bridge(name = "cancel-task", lib = "(koru-task)")]
pub async fn cancel_task(id: &Value) -> Result<Vec<Value>, Exception> {
    let id: Number = id.clone().try_into()?;
    let id: usize = id.try_into()?;
    TaskManager::cancel_task(id).await;
    Ok(Vec::new())
}

#[bridge(name = "cancel-tasks", lib = "(koru-task)")]
pub async fn cancel_tasks() -> Result<Vec<Value>, Exception> {
    TaskManager::cancel_all_tasks().await;
    Ok(Vec::new())
}