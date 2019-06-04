use std::sync::Arc;
use parking_lot::{Mutex,Condvar};
use std::collections::VecDeque;

// type WorkQueue = Arc<Mutex<(VecDeque<Box<Fn()+Send>>, Condvar)>>;

pub struct TaskQueue {
    queue: VecDeque<Box<FnOnce()+Send>>,
}
impl TaskQueue {
    pub fn new() -> Self {
        Self {queue: VecDeque::new()}
    }
    pub fn enqueue<F: 'static>(&mut self, task: F) where F: FnOnce()+Send {
        self.queue.push_back(Box::new(task));
    }
}

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

    pub fn enqueue<F: 'static>(&self, f: F) where F: FnOnce()+Send {
        let mut queue_lock = self.work.tasks.lock();
        queue_lock.push_back(Box::new(f));
        drop(queue_lock);
        self.work.cvar.notify_one();
    }
    pub fn enqueue_many(&self, mut vf: TaskQueue) {
        let mut queue_lock = self.work.tasks.lock();
        queue_lock.append(&mut vf.queue);
        drop(queue_lock);
        self.work.cvar.notify_all();
    }
    pub fn shutdown(&self) {
        let mut shutdown_lock = self.work.shutdown.lock();
        *shutdown_lock = true;
        self.work.cvar.notify_all();
    }
    pub fn shutdown_wait(&mut self) {
        self.shutdown();
        for _i in 0..self.threads.len() {
            self.threads.pop().unwrap().join().expect("Error joining thread!");
        }
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn worker(queue: Arc<WorkQueue>) {
    loop {
        let mut queue_lock = queue.tasks.lock();

        while queue_lock.len() == 0 {
            if *queue.shutdown.lock() {
                // No more tasks and pool is shutting down. Stop worker.
                return;
            }
            queue.cvar.wait(&mut queue_lock);
        }
        let task = queue_lock.pop_front().unwrap();
        drop(queue_lock);

        task();
    }
}