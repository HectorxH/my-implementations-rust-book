use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

type JoinHandle = thread::JoinHandle<()>;
type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new `ThreadPool`.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// Will panic if `n_workers` is 0.
    ///
    /// # Examples
    /// ```
    /// use hello::ThreadPool;
    ///
    /// // Create a ThreadPool with 4 workers
    /// let mut pool = ThreadPool::new(4);
    /// ```
    pub fn new(n_workers: usize) -> ThreadPool {
        assert!(n_workers > 0);

        let (tx, rx) = mpsc::channel();

        let rx = Arc::new(Mutex::new(rx));

        let mut workers = Vec::with_capacity(n_workers);

        for id in 0..n_workers {
            workers.push(Worker::new(id, Arc::clone(&rx)));
        }

        return ThreadPool {
            workers,
            sender: Some(tx),
        };
    }

    /// Executes the given `Job` on a free worker from the `ThreadPool`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hello::ThreadPool;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let mut pool = ThreadPool::new(4);
    /// let counter = Arc::new(Mutex::new(0));
    ///
    /// for _ in (0..4) {
    ///     let counter = Arc::clone(&counter);
    ///     pool.execute(move || {
    ///         let mut counter = counter.lock().unwrap();
    ///         *counter += 1;
    ///     });
    /// }
    /// ```
    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // No sender means we are shutting down the server,
        // so no new Jobs should be sent.
        if let Some(sender) = &self.sender {
            // Sends returns an error only of the receiver got
            // dropped, we can't recover from that state.
            sender.send(Box::new(f)).unwrap();
        }
    }
}

impl Drop for ThreadPool {
    /// Drops the communication with it's threads, stoping idle
    /// threads in the process. Waits for remaining threads to finish
    /// their jobs.
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread
                    .join()
                    .expect("A thread panicked while calling `join`.")
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle>,
}

impl Worker {
    /// Create a new `Worker` that will wait for tasks while it has a `thread`.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // Unwrap to panic in case another thread panicked.
            let receiver = receiver.lock().unwrap();
            let message = receiver.recv();

            drop(receiver);

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    job();
                    println!("Worker {id} finished job; waiting.");
                }
                Err(_) => {
                    println!("Worker {id} discconected; shutting down.");
                    break;
                }
            }
        });

        return Worker {
            id,
            thread: Some(thread),
        };
    }
}
