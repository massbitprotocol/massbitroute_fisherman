/*
 * Check from any gateway can connection to any node
 */

use crate::models::jobs::JobAssignmentBuffer;
use crate::tasks::generator::TaskApplicant;
use common::component::ComponentInfo;
use common::job_manage::JobRole;
use common::workers::MatchedWorkers;
use common::{Gateway, Node, PlanId};
use std::sync::Arc;

pub struct TaskGWNodeConnection {
    list_nodes: Vec<Arc<Node>>,
    list_gateways: Vec<Arc<Gateway>>,
}
impl TaskGWNodeConnection {
    pub fn get_name() -> String {
        String::from("TaskGWNode")
    }

    pub fn new() -> Self {
        TaskGWNodeConnection {
            list_nodes: vec![],
            list_gateways: vec![],
        }
    }
}
impl TaskApplicant for TaskGWNodeConnection {
    fn get_name(&self) -> String {
        Self::get_name()
    }
    fn can_apply(&self, _component: &ComponentInfo) -> bool {
        true
    }

    fn apply(
        &self,
        _plan_id: &PlanId,
        _component: &ComponentInfo,
        _phase: JobRole,
        _workers: &MatchedWorkers,
    ) -> Result<JobAssignmentBuffer, anyhow::Error> {
        Ok(JobAssignmentBuffer::default())
    }
}
