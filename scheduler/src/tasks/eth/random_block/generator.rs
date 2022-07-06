use crate::models::jobs::AssignmentBuffer;
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
    fn can_apply(&self, component: &ComponentInfo) -> bool {
        true
    }

    fn apply(
        &self,
        plan_id: &PlanId,
        node: &Node,
        phase: JobRole,
        workers: &MatchedWorkers,
    ) -> Result<AssignmentBuffer, Error> {
        Ok(AssignmentBuffer::default())
    }
}
