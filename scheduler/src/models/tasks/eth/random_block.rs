use crate::models::tasks::TaskApplicant;
use anyhow::Error;
use common::component::ComponentInfo;
use common::job_manage::{Job, JobDetail, JobPing, JobResult};
use common::{Gateway, Node};
use std::sync::Arc;
use std::vec;

/*
 * Apply for node to get randome block number and response time
 */
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct TaskRandomBlock {
    assigned_at: u64, //Timestamp to assign job
    finished_at: u64, //Timestamp when result has arrived
}

impl TaskRandomBlock {
    pub fn new() -> Self {
        TaskRandomBlock {
            assigned_at: 0,
            finished_at: 0,
        }
    }
}

impl TaskApplicant for TaskRandomBlock {
    fn apply(&self, node: Arc<Node>) -> Result<Vec<Job>, Error> {
        let vec = Vec::default();
        Ok(vec)
    }
}
