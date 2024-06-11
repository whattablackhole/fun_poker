use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
};

pub mod lobby;

pub mod card;
pub mod dealer;
pub mod player;
pub mod postgres_database;
pub mod socket_pool;
pub mod dealer_pool;

pub mod protos {
    pub mod client_state {
        include!("protos_rs/client_state.rs");
    }

    pub mod lobby {
        include!("protos_rs/lobby.rs");
    }

    pub mod game_state {
        include!("protos_rs/game_state.rs");
    }

    pub mod empty {
        include!("protos_rs/empty.rs");
    }

    pub mod player {
        include!("protos_rs/player.rs");
    }

    pub mod user {
        include!("protos_rs/user.rs");
    }

    pub mod card {
        include!("protos_rs/card.rs");
    }

    pub mod client_request {
        include!("protos_rs/client_request.rs");
    }
}

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
