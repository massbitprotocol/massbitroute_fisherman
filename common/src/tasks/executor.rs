use crate::jobs::{Job, JobResult};
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
