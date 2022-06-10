use crate::component::ComponentInfo;
use crate::job_manage::{Job, JobDetail, JobPing, JobResult};
use crate::models::PlanEntity;
use crate::tasks::generator::TaskApplicant;
use crate::{Gateway, Node};
use anyhow::Error;
use serde::{Deserialize, Serialize};
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
    fn can_apply(&self, component: &ComponentInfo) -> bool {
        true
    }

    fn apply(&self, plan: &PlanEntity, node: &Node) -> Result<Vec<Job>, Error> {
        let vec = Vec::default();
        Ok(vec)
    }
}
