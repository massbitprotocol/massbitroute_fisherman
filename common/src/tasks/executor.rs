use crate::job_manage::JobResult;
use crate::jobs::Job;
use crate::tasks::eth::benchmark::executor::BenchmarkExecutor;
use crate::tasks::eth::latest_block::executor::LatestBlockExecutor;
use crate::tasks::ping::executor::PingExecutor;
use crate::util::get_current_time;
use crate::WorkerId;
use async_trait::async_trait;
use log::{debug, info};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[async_trait]
pub trait TaskExecutor: Sync + Send {
    async fn execute(
        &self,
        job: &Job,
        result_sender: Sender<JobResult>,
    ) -> Result<(), anyhow::Error>;
    fn can_apply(&self, job: &Job) -> bool;
}

pub fn get_executors(worker_id: WorkerId, benchmark_wrk_path: &str) -> Vec<Arc<dyn TaskExecutor>> {
    let mut result: Vec<Arc<dyn TaskExecutor>> = Default::default();
    result.push(Arc::new(PingExecutor::new(worker_id.clone())));
    result.push(Arc::new(BenchmarkExecutor::new(
        worker_id.clone(),
        benchmark_wrk_path,
    )));
    result.push(Arc::new(LatestBlockExecutor::new(worker_id.clone())));
    result
}
