use crate::job_manage::{Job, JobResult};
use crate::tasks::eth::benchmark::executor::BenchmarkExecutor;
use crate::tasks::ping::executor::PingExecutor;
use crate::util::get_current_time;
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
    async fn generate_new_job(&self, job: &Job, newjob_sender: Sender<Job>) {
        let mut job = job.clone();
        if job.repeat_number > 0 {
            job.expected_runtime = get_current_time() + job.interval;
            job.repeat_number = job.repeat_number - 1;
            debug!("Schedule new repeat job: {:?}", job);
            newjob_sender.send(job).await;
        }
    }
    fn can_apply(&self, job: &Job) -> bool;
}

pub fn get_executors(benchmark_wrk_path: &str) -> Vec<Arc<dyn TaskExecutor>> {
    let mut result: Vec<Arc<dyn TaskExecutor>> = Default::default();
    result.push(Arc::new(PingExecutor::new()));
    result.push(Arc::new(BenchmarkExecutor::new(benchmark_wrk_path)));
    result
}
