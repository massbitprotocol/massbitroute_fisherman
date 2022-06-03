use crate::executor::TaskExecutor;
use anyhow::Error;
use async_trait::async_trait;
use common::component::ComponentInfo;
use common::job_manage::{Job, JobDetail, JobPing, JobPingResult, JobResult};
use common::{Gateway, Node};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::vec;
use tokio::sync::mpsc::Sender;
/*
 * Periodically ping to node/gateway to get response time, to make sure node/gateway is working
 */
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct TaskPing {
    assigned_at: u64, //Timestamp to assign job
    finished_at: u64, //Timestamp when result has arrived
}

impl TaskPing {
    pub fn new() -> Self {
        TaskPing {
            assigned_at: 0,
            finished_at: 0,
        }
    }
}
#[async_trait]
impl TaskExecutor for TaskPing {
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
