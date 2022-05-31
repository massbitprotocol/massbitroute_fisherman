use crate::models::jobs::AssignmentBuffer;
use crate::models::providers::ProviderStorage;
use crate::models::tasks::{get_tasks, TaskApplicant};
use crate::models::workers::WorkerPool;
use common::component::ComponentInfo;
use common::job_manage::Job;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct JobGenerator {
    providers: Arc<Mutex<ProviderStorage>>,
    workers: Arc<WorkerPool>,
    tasks: Vec<Arc<dyn TaskApplicant>>,
    job_assignments: Arc<AssignmentBuffer>,
}

impl JobGenerator {
    pub fn new(providers: Arc<Mutex<ProviderStorage>>, workers: Arc<WorkerPool>) -> Self {
        let tasks = get_tasks();
        let job_assignments = Arc::new(AssignmentBuffer::default());
        JobGenerator {
            providers,
            workers,
            tasks,
            job_assignments,
        }
    }
    pub fn init(&mut self) {
        loop {
            println!("Generator jobs");
            self.generate_jobs();
            sleep(Duration::from_secs(60));
        }
    }
    pub async fn generate_jobs(&mut self) {
        let providers = self.providers.clone();
        for task in self.tasks.iter() {
            providers.lock().await.apply_task(task)
        }
    }
}
