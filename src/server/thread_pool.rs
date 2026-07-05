use std::sync::{Arc,Mutex};
use std::thread;
use std::sync::mpsc::{self,Sender,Receiver};

pub struct ThreadPool {
    worker : Vec<Worker>,
    sender : Option<Sender<Job>>,
    is_shutdown : bool,
}

type Job = Box<dyn fnOnce() + Send + 'static>;

impl ThreadPool {

    pub fn new(size:usize)-> Self{
        assert!(size>0,"Thread pool size must be greater than 0");
        
        let(sender,receiver) = mpsc::channel();
        let receiver = Arc::new(Mutes::new(receiver));
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
    }
    
    pub fn execute<F>(&self,f:F)->Result<(),ThreadPoolError>
        where
            F : FnOnce() + Send + 'static,{
                if self.is_shutdown {
                    return Err(ThreadPoolError::shutdown);
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

}
