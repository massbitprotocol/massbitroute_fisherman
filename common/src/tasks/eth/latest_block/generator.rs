use crate::component::ComponentInfo;
use crate::job_manage::{Job, JobDetail, JobPing, JobResult};
use crate::models::PlanEntity;
use crate::tasks::eth::{JobLatestBlock, JobLatestBlockResult};
use crate::tasks::generator::TaskApplicant;
use crate::{Node, PlanId};
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
    pub fn create_url(&self, component: &ComponentInfo) -> String {
        format!("https://{}/", component.ip)
    }
}

impl TaskApplicant for TaskLatestBlock {
    fn can_apply(&self, component: &ComponentInfo) -> bool {
        true
    }

    fn apply(&self, plan_id: &PlanId, node: &Node) -> Result<Vec<Job>, Error> {
        let job = JobLatestBlock {
            assigned_at: 0,
            finished_at: 0,
        };
        let job_detail = JobDetail::LatestBlock(job);
        let mut job = Job::new(plan_id.clone(), job_detail.get_job_name(), node, job_detail);
        job.parallelable = true;
        job.component_url = self.create_url(node);
        let vec = vec![job];
        Ok(vec)
    }
}
