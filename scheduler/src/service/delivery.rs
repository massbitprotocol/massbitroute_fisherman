use crate::models::workers::WorkerPool;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

#[derive(Default)]
pub struct JobDelivery {
    worker_pool: Arc<WorkerPool>,
}

impl JobDelivery {
    pub fn new(worker_pool: Arc<WorkerPool>) -> Self {
        JobDelivery {
            worker_pool: Arc::new(Default::default()),
        }
    }
    pub fn init(&mut self) {
        loop {
            println!("Delivery jobs");
            sleep(Duration::from_secs(60));
        }
    }
}
