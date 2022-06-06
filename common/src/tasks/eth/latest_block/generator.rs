use crate::component::ComponentInfo;
use crate::job_manage::{Job, JobDetail, JobPing, JobResult};
use crate::tasks::generator::TaskApplicant;
use crate::Node;
use anyhow::Error;
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
    fn can_apply(&self, component: &ComponentInfo) -> bool {
        true
    }

    fn apply(&self, node: &Node) -> Result<Vec<Job>, Error> {
        let vec = Vec::default();
        Ok(vec)
    }
}
