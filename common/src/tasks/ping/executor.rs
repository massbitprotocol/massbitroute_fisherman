use crate::job_manage::{Job, JobPingResult, JobResult};
use crate::tasks::executor::TaskExecutor;
use anyhow::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct PingExecutor {}

impl PingExecutor {
    pub fn new() -> Self {
        PingExecutor {}
    }
}

#[async_trait]
impl TaskExecutor for PingExecutor {
    async fn execute(&self, job: &Job, sender: Sender<JobResult>) -> Result<JobResult, Error> {
        log::debug!("TaskPing execute for job {:?}", &job);
        let ping_result = JobPingResult {
            job: Default::default(),
            response_timestamp: 0,
            response_time: vec![],
        };
        let result = JobResult::Ping(ping_result);
        Ok(result)
    }
}
