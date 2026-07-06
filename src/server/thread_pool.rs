use std::sync::{Arc,Mutex};
use std::thread;
use std::sync::mpsc::{self,Sender,Receiver};

pub struct ThreadPool {
    workers : Vec<Worker>,
    sender : Option<Sender<Job>>,
    is_shutdown : bool,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {

    pub fn new(size:usize)-> Self{
        assert!(size>0,"Thread pool size must be greater than 0");
        
        let(sender,receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id,Arc::clone(&receiver)));
        }
        ThreadPool{
            workers,
            sender : Some(sender),
            is_shutdown : false,
        }
    }

    pub fn with_default_size() -> Self{
        let cores = num_cpus::get();
        println!("Creating thread pool with {} workers ",cores);
        Self::new(cores)
    }
    
    pub fn execute<F>(&self,f:F)->Result<(),ThreadPoolError>
        where
            F : FnOnce() + Send + 'static,{
                if self.is_shutdown {
                    return Err(ThreadPoolError::Shutdown);
                }
                let job = Box::new(f);
                match self.sender.as_ref() {
                    Some(sender) => {
                        sender.send(job)
                            .map_err(|_| ThreadPoolError::SendError)?;
                        Ok(())
                    }
                    None => Err(ThreadPoolError::Shutdown),
                }
    }

    pub fn shutdown(mut self)->Result<(),ThreadPoolError>{
        println!("Shutting down the thread pool.....");

        drop(self.sender.take());
        self.is_shutdown = true;

        for worker in &mut self.workers{
            println!("Waiting for worker {} to finish",worker.id);
            if let Some(thread) = worker.thread.take(){
                thread.join().map_err(|_| ThreadPoolError::JoinError)?;
            }
            println!("Thread Pool shutdown completeled");
        }
            Ok(())
    }

    pub fn worker_count(&self)->usize{
        self.workers.len()
    }

    pub fn is_shutdown(&self)->bool{
        self.is_shutdown
    }
}


impl Drop for ThreadPool {
    fn drop(&mut self){
        if !self.is_shutdown{
            println!("Threadpool dropped without shutdown");
            drop(self.sender.take());
            for workers in &mut self.workers{
                if let Some(thread) = workers.thread.take(){
                    let _ = thread.join();  
                }
            }
        }
    }
}
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
impl Worker {
    fn new(id:usize,receiver:Arc<Mutex<Receiver<Job>>>) -> Self{
        let thread = thread::spawn(move || {
            println!("Worker {} started",id);
            loop{
                let job={
                    let receiver_guard = receiver.lock().unwrap();
                    receiver_guard.recv()
                };
                match job{
                    Ok(job)=>{
                        println!("Worker executing job {}",id);
                        job();
                        println!("Worker completed job {}",id);
                    }
                    Err(_)=>{
                        println!("Worker {} Stopped",id);
                    }
                }
            }
            println!("Worker thread stopped {}",id);
        });

        Worker{
            id,
            thread:Some(thread),
        }
    }
}

#[derive(Debug,Clone,PartialEq)]
pub enum ThreadPoolError{
    Shutdown,
    SendError,
    JoinError
}

impl std::fmt::Display for ThreadPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadPoolError::Shutdown => write!(f, "Thread pool is shut down"),
            ThreadPoolError::SendError => write!(f, "Failed to send job to worker"),
            ThreadPoolError::JoinError => write!(f, "Failed to join worker thread"),
        }
    }
}
impl std::error::Error for ThreadPoolError{}
