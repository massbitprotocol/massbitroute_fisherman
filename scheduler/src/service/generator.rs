use crate::models::jobs::AssignmentBuffer;
use crate::models::providers::ProviderStorage;
use crate::models::tasks::{get_tasks, TaskApplicant};
use crate::models::workers::WorkerPool;
use common::component::ComponentInfo;
use common::job_manage::Job;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

#[derive(Default)]
pub struct JobGenerator {
    providers: Arc<ProviderStorage>,
    workers: Arc<WorkerPool>,
    tasks: Vec<Arc<dyn TaskApplicant>>,
    job_assignments: Arc<AssignmentBuffer>,
}

impl JobGenerator {
    pub fn new(
        providers: Arc<ProviderStorage>,
        workers: Arc<WorkerPool>,
        job_assignments: Arc<AssignmentBuffer>,
    ) -> Self {
        let tasks = get_tasks();
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

            sleep(Duration::from_secs(60));
        }
    }
    pub fn generate_node_jobs() {}
    pub fn generate_gateway_jobs() {}
    pub fn generate_checkpath_job_for_nodes(
        &self,
        componentInfo: ComponentInfo,
    ) -> Result<Vec<Job>, anyhow::Error> {
        Ok(Vec::default())
    }
    pub fn generate_checkpath_job_for_gateways(
        &self,
        componentInfo: ComponentInfo,
    ) -> Result<Vec<Job>, anyhow::Error> {
        Ok(Vec::default())
    }

    /*
     * for benchmarking job need more preparation:
     * 1 - quarantine node/gateway from MBR network
     * 2 - Send job to worker to execute
     * 3 - Check result and rejoin node/gateway to MBR network
     * Periodically
     */

    pub fn generate_benchmark_for_nodes() -> Result<Vec<Job>, anyhow::Error> {
        Ok(Vec::default())
    }
    pub fn generate_benchmark_for_gateways() -> Result<Vec<Job>, anyhow::Error> {
        Ok(Vec::default())
    }
}
