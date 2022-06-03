pub mod ping;

pub mod dot;
pub mod eth;

use crate::executor::ping::TaskPing;
use async_trait::async_trait;
use common::job_manage::{Job, JobResult};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[async_trait]
pub trait TaskExecutor: Sync + Send {
    async fn execute(
        &self,
        job: &Job,
        sender: Sender<JobResult>,
    ) -> Result<JobResult, anyhow::Error>;
}
pub fn get_eth_executors() -> Vec<Arc<dyn TaskExecutor>> {
    let mut result: Vec<Arc<dyn TaskExecutor>> = Default::default();
    result.push(Arc::new(TaskPing::new()));
    result
}
