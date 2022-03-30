use super::{ProcessControlBlock, TaskControlBlock};
use crate::sync::UPSafeCell;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use lazy_static::*;

pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple SPN scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        let task_inner = task.inner_exclusive_access();
        let prediction = task_inner.task_prediction;
        let running = task_inner.task_isrunning;
        // if prediction != 1000000{
        //     println!("{}", prediction);
        // }
        drop(task_inner);
        for queue in 0..self.ready_queue.len(){
            let task1 = self.ready_queue.get_mut(queue).unwrap();
            let prediction1 = task1.inner_exclusive_access().task_prediction;
            let running1 = task1.inner_exclusive_access().task_isrunning;
            if running && !running1{
                self.ready_queue.insert(queue, task);
                return
            }
            else if !running && running1{
                continue;
            }
            else if prediction < prediction1 {
                // if prediction != 1000000{
                //     println!("{},{}", prediction,prediction1);
                // }
                self.ready_queue.insert(queue, task);
                return
            }
        }
        self.ready_queue.push_back(task);
    }
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
    pub static ref PID2PCB: UPSafeCell<BTreeMap<usize, Arc<ProcessControlBlock>>> =
        unsafe { UPSafeCell::new(BTreeMap::new()) };
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}

pub fn pid2process(pid: usize) -> Option<Arc<ProcessControlBlock>> {
    let map = PID2PCB.exclusive_access();
    map.get(&pid).map(Arc::clone)
}

pub fn insert_into_pid2process(pid: usize, process: Arc<ProcessControlBlock>) {
    PID2PCB.exclusive_access().insert(pid, process);
}

pub fn remove_from_pid2process(pid: usize) {
    let mut map = PID2PCB.exclusive_access();
    if map.remove(&pid).is_none() {
        panic!("cannot find pid {} in pid2task!", pid);
    }
}
