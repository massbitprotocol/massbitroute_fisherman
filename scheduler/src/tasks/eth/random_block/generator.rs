use crate::models::jobs::JobAssignmentBuffer;
use crate::tasks::generator::TaskApplicant;
use anyhow::Error;
use common::component::ComponentInfo;
use common::job_manage::JobRole;
use common::workers::MatchedWorkers;
use common::{Node, PlanId};
use serde::{Deserialize, Serialize};

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
    fn get_name(&self) -> String {
        String::from("RandomBlock")
    }
    fn can_apply(&self, _component: &ComponentInfo) -> bool {
        true
    }

    fn apply(
        &self,
        _plan_id: &PlanId,
        _node: &Node,
        _phase: JobRole,
        _workers: &MatchedWorkers,
    ) -> Result<JobAssignmentBuffer, Error> {
        Ok(JobAssignmentBuffer::default())
    }
}
