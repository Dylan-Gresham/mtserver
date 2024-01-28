use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

/// ThreadPool struct
///
/// # Members
///
/// - `workers` A vec containing all the Workers
/// - `sender` A Sender to send Jobs to Workers
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

/// Worker struct
///
/// # Members
///
/// - `id` The id representing this Worker
/// - `thread` The thread running the Job
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// - `size` is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if size is less than or equal to 0.
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));

        for i in 0..size {
            workers.push(Worker::new(i, Arc::clone(&rx)));
        }

        Self {
            workers,
            sender: Some(tx),
        }
    }

    /// Sends the job into the channel for a Worker to execute.
    ///
    /// # Arguments
    ///
    /// - `f` is the function to be executed.
    ///
    /// # Panics
    ///
    /// Awaiting proper handling.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    /// Create a new worker.
    ///
    /// # Arguments
    ///
    /// - `id` is the ID corresponding to this Worker.
    /// - `receiver` is the channel receiver for the Worker to get it's Job from.
    ///
    /// # Return
    ///
    /// A new Worker struct
    ///
    /// # Panics
    ///
    /// Awaiting explicit handling.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver
                .lock()
                .expect("Worker: {id} - The receiver was poisoned!")
                .recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }
}
