use crate::models::providers::ProviderStorage;
use crate::models::workers::WorkerPool;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

#[derive(Default)]
pub struct JobGenerator {
    providers: Arc<ProviderStorage>,
    workers: Arc<WorkerPool>,
}

impl JobGenerator {
    pub fn new(providers: Arc<ProviderStorage>, workers: Arc<WorkerPool>) -> Self {
        JobGenerator { providers, workers }
    }
    pub fn init(&mut self) {
        loop {
            println!("Generator jobs");
            sleep(Duration::from_secs(60));
        }
    }
}
