use crate::task::Task;
use alloc::collections::VecDeque;
use core::task::{Context, Poll, Waker, RawWaker, RawWakerVTable};

pub struct Executor {
    task_queue: VecDeque<Task>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            task_queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task);
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            x86_64::instructions::hlt(); 
        }
    }

    fn run_ready_tasks(&mut self) {
        let mut remaining_tasks = self.task_queue.len();
        
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            
            match task.future.as_mut().poll(&mut context) {
                Poll::Ready(()) => {} // Tâche terminée, on ne la repousse pas
                Poll::Pending => {
                    self.task_queue.push_back(task); // Pas fini, on la remet en queue
                }
            }

            remaining_tasks -= 1;
            if remaining_tasks == 0 { break; }
        }
    }
}

fn dummy_waker() -> Waker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker { RawWaker::new(core::ptr::null(), &VTABLE) }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no_op, no_op, no_op);
    unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE)) }
}