use std::sync::{Arc,Mutex,Condvar};
use std::collections::VecDeque;

// type WorkQueue = Arc<Mutex<(VecDeque<Box<Fn()+Send>>, Condvar)>>;


struct WorkQueue {
    tasks: Mutex<VecDeque<Box<FnOnce()+Send>>>,
    cvar: Condvar,
    shutdown: Mutex<bool>,
}
impl WorkQueue {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(VecDeque::new()),
            cvar: Condvar::new(),
            shutdown: Mutex::new(false),
        }
    }
}

pub struct ThreadPool {
    threads: Vec<std::thread::JoinHandle<()>>,
    work: Arc<WorkQueue>,  
}
impl ThreadPool {
    pub fn new(thread_num: usize) -> Self {
        let mut threads = Vec::with_capacity(thread_num);
        let work = Arc::new(WorkQueue::new());
        for _i in 0..thread_num {
            let threadwork = Arc::clone(&work);
            threads.push(std::thread::spawn(move || {
                worker(threadwork);
            }));
        }

        Self {threads, work}
    }

    // TODO: stop the threads when the pool gets destroyed.

    pub fn enqueue(&self, f: Box<FnOnce()+Send>) {
        let mut queue_lock = self.work.tasks.lock().unwrap();
        queue_lock.push_back(f);
        drop(queue_lock);
        self.work.cvar.notify_one();
    }
    pub fn shutdown(&self) {
        let mut shutdown_lock = self.work.shutdown.lock().unwrap();
        *shutdown_lock = true;
    }
}

fn worker(queue: Arc<WorkQueue>) {
    loop {
        let mut queue_lock = queue.tasks.lock().unwrap();

        while queue_lock.len() == 0 {
            if *queue.shutdown.lock().unwrap() {
                // No more tasks and pool is shutting down. Stop worker.
                return;
            }
            queue_lock = queue.cvar.wait(queue_lock).unwrap();
        }
        let task = queue_lock.pop_front().unwrap();
        drop(queue_lock);

        task();
    }
}