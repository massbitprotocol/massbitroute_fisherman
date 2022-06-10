/*
 * Check from any gateway can connection to any node
 */

use crate::component::ComponentInfo;
use crate::job_manage::Job;
use crate::models::PlanEntity;
use crate::tasks::generator::TaskApplicant;
use crate::{Gateway, Node};
use std::sync::Arc;

pub struct TaskGWNodeConnection {
    list_nodes: Vec<Arc<Node>>,
    list_gateways: Vec<Arc<Gateway>>,
}
impl TaskGWNodeConnection {
    pub fn new() -> Self {
        TaskGWNodeConnection {
            list_nodes: vec![],
            list_gateways: vec![],
        }
    }
}
impl TaskApplicant for TaskGWNodeConnection {
    fn can_apply(&self, component: &ComponentInfo) -> bool {
        true
    }

    fn apply(
        &self,
        plan: &PlanEntity,
        component: &ComponentInfo,
    ) -> Result<Vec<Job>, anyhow::Error> {
        let vec = Vec::default();
        Ok(vec)
    }
}
