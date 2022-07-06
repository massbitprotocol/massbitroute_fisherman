use crate::job_manage::JobResultDetail;
use crate::jobs::{Job, JobResult};
use crate::tasks::eth::benchmark::executor::BenchmarkExecutor;
use crate::tasks::eth::latest_block::executor::LatestBlockExecutor;
use crate::tasks::ping::executor::PingExecutor;
use crate::util::get_current_time;
use crate::WorkerId;
use async_trait::async_trait;
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
