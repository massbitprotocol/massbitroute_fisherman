/*
 * Check from any gateway can connection to any node
 */

use crate::models::tasks::TaskApplicant;
use common::component::ComponentInfo;
use common::job_manage::Job;
use common::{Gateway, Node};
use std::sync::Arc;

pub struct TaskGWNodeConnection {
    list_nodes: Vec<Arc<Node>>,
    list_gateways: Vec<Act<Gateway>>,
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
    fn apply(&self, component: Arc<ComponentInfo>) -> Result<Vec<Job>, anyhow::Error> {
        todo!()
    }
}
