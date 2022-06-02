use crate::models::tasks::TaskApplicant;
use anyhow::Error;
use common::component::ComponentInfo;
use common::job_manage::{Job, JobDetail, JobPing, JobResult};
use common::{Gateway, Node};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::vec;
/*
 * Apply for node to get latest block number and time
 */
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct TaskLatestBlock {
    assigned_at: u64, //Timestamp to assign job
    finished_at: u64, //Timestamp when result has arrived
}

impl TaskLatestBlock {
    pub fn new() -> Self {
        TaskLatestBlock {
            assigned_at: 0,
            finished_at: 0,
        }
    }
}

impl TaskApplicant for TaskLatestBlock {
    fn apply(&self, node: &Node) -> Result<Vec<Job>, Error> {
        let vec = Vec::default();
        Ok(vec)
    }
}
