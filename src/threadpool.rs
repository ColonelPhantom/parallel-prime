use parking_lot::RwLock;
use crossbeam_channel::bounded;

pub struct ThreadPool {
    threads: Vec<std::thread::JoinHandle<()>>,
    tx: Option<crossbeam_channel::Sender<Box<FnOnce()+Send>>>,
    shutdown: RwLock<bool>,
}
impl ThreadPool {
    pub fn new(thread_num: usize, queue_size: usize) -> Self {
        let mut threads = Vec::with_capacity(thread_num);
        let (tx, rx) = bounded(queue_size);
        for _i in 0..thread_num {
            let wrx = rx.clone();
            threads.push(std::thread::spawn(move || {
                worker(wrx);
            }));
        }
        let shutdown = RwLock::new(false);

        Self {threads, tx: Some(tx), shutdown}
    }

    pub fn enqueue<F: 'static>(&self, f: F) where F: FnOnce()+Send {
        match &self.tx {
            Some(tx) => tx.send(Box::new(f)).unwrap(),
            None => match *self.shutdown.read() {
                true => panic!("Attempting to enqueue into a shutdown pool!"),
                false => panic!("Transmitter in pool is None for unknown reason. Please report a bug."),
            },
        }
    }
    pub fn shutdown(&mut self) {
        let mut shutdown_lock = self.shutdown.write();
        *shutdown_lock = true;
        drop(shutdown_lock);
        self.tx = None;
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

fn worker(queue: crossbeam_channel::Receiver<Box<FnOnce()+Send>>) {
    loop {
        let task = queue.recv();
        match task {
            Ok(t) => t(),
            Err(_e) => return,
        }
    }
}