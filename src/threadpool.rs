use std::sync::{Arc,Mutex,Condvar,MutexGuard};
use std::collections::VecDeque;

// type WorkQueue = Arc<Mutex<(VecDeque<Box<Fn()+Send>>, Condvar)>>;


struct WorkQueue {
    tasks: Mutex<VecDeque<Box<FnOnce()+Send>>>,
    cvar: Condvar,
}
impl WorkQueue {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(VecDeque::new()),
            cvar: Condvar::new(),
        }
    }
}

pub struct ThreadPool {
    threads: Vec<std::thread::Thread>,
    work: Arc<WorkQueue>,
    
}
impl ThreadPool {
    pub fn new(thread_num: usize) -> Self {
        let mut threads = Vec::with_capacity(thread_num);
        let work = Arc::new(WorkQueue::new());
        for i in 0..thread_num {
            let threadwork = Arc::clone(&work);
            threads[i] = std::thread::spawn(move || {
                worker(threadwork);
            }).thread().clone();
        }

        Self {threads, work}
    }

    pub fn enqueue(f: Box<FnOnce()+Send>) {

    }
}

fn worker(queue: Arc<WorkQueue>) {
    loop {
        let mut queue_lock: MutexGuard<VecDeque<Box<FnOnce()+Send>>> = queue.tasks.lock().unwrap();

        while queue_lock.len() == 0 {
            queue_lock = queue.cvar.wait(queue_lock).unwrap();
        }
        let task = queue_lock.pop_front().unwrap();
        drop(queue_lock);

        task();
    }
}