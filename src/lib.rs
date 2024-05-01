use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
};

pub mod lobby;

pub mod player;

pub mod socket_pool;

pub mod postgres_database;
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<()>>,
}
impl Worker {
    fn new(id: usize, job: Arc<Mutex<Receiver<Job>>>) -> Worker {

        let handle = thread::spawn(move || loop {
            let message = job.lock().unwrap().recv();

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

        let worker = Worker {
            id,
            handle: Some(handle),
        };
        worker
    }
}
type Job = Box<dyn FnOnce() + Send + 'static>;

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.handle.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl ThreadPool {

    pub fn new(quantity: usize) -> Self {
        assert!(quantity > 0);
        let (sender, receiver) = mpsc::channel();

        let mut workers = Vec::with_capacity(quantity);

        let rc = Arc::new(Mutex::new(receiver));

        for i in 0..quantity {
            workers.push(Worker::new(i, Arc::clone(&rc)))
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.as_ref().unwrap().send(Box::new(f)).unwrap();
    }
}
