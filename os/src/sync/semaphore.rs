//! Semaphore

use crate::sync::UPSafeCell;
use crate::task::{block_current_and_run_next, current_task, wakeup_task, TaskControlBlock, current_process};
use alloc::{collections::VecDeque, sync::Arc};

/// semaphore structure
pub struct Semaphore {
    /// semaphore inner
    pub inner: UPSafeCell<SemaphoreInner>,
}

pub struct SemaphoreInner {
    pub count: isize,
    pub wait_queue: VecDeque<Arc<TaskControlBlock>>,
    pub id: usize,
}

impl Semaphore {
    /// Create a new semaphore
    pub fn new(res_count: usize, id: usize) -> Self {
        trace!("kernel: Semaphore::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(SemaphoreInner {
                    count: res_count as isize,
                    wait_queue: VecDeque::new(),
                    id
                })
            },
        }
    }

    /// up operation of semaphore
    pub fn up(&self) {
        trace!("kernel: Semaphore::up");
        let mut inner = self.inner.exclusive_access();
        inner.count += 1;

        let process = current_process();
        let mut process_inner = process.inner_exclusive_access();
        process_inner.semaphore_available[inner.id] += 1;

        if inner.count <= 0 {
            if let Some(task) = inner.wait_queue.pop_front() {
                // let task_inner = task.inner_exclusive_access();
                // let tid = task_inner.res.as_ref().unwrap().tid;
                // process_inner.semaphore_need[tid][inner.id] -= 1;
                // drop(task_inner);
                drop(process_inner);
                wakeup_task(task);
            }
        }
    }

    /// down operation of semaphore
    pub fn down(&self) {
        trace!("kernel: Semaphore::down");
        let mut inner = self.inner.exclusive_access();
        inner.count -= 1;
        
        // process_inner.semaphore_available[inner.id] -= 1;
        
        if inner.count < 0 {
            inner.wait_queue.push_back(current_task().unwrap());
            
            let process = current_process();
            let mut process_inner = process.inner_exclusive_access();
            let task = current_task().unwrap();
            let task_inner = task.inner_exclusive_access();
            let tid = task_inner.res.as_ref().unwrap().tid;
            process_inner.semaphore_need[tid][inner.id] += 1;
            
            drop(task_inner);
            drop(process_inner);
            drop(inner);
            block_current_and_run_next();
        }
    }
}
