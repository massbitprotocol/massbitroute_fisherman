/*
 * Check from any gateway can connection to any node
 */

use crate::tasks::generator::TaskApplicant;
use common::component::ComponentInfo;
use common::jobs::Job;
use common::{Gateway, Node, PlanId};
use std::sync::Arc;

pub struct TaskGWNodeConnection {
    list_nodes: Vec<Arc<Node>>,
    list_gateways: Vec<Arc<Gateway>>,
}
impl TaskGWNodeConnection {
    pub fn get_name() -> String {
        String::from("GatewayNodeConnection")
    }
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
        plan_id: &PlanId,
        component: &ComponentInfo,
    ) -> Result<Vec<Job>, anyhow::Error> {
        let vec = Vec::default();
        Ok(vec)
    }
}
