/*
 * Check from any gateway can connection to any node
 */

use crate::models::tasks::TaskApplicant;
use common::component::ComponentInfo;
use common::job_manage::Job;
use common::{Gateway, Node};
use std::sync::Arc;

pub struct NodeBenchmark {}

impl NodeBenchmark {
    pub fn new() -> Self {
        NodeBenchmark {}
    }
}
impl TaskApplicant for NodeBenchmark {
    fn apply(&self, node: &Node) -> Result<Vec<Job>, anyhow::Error> {
        Ok(Vec::new())
    }
}

pub struct GatewayBenchmark {}

impl GatewayBenchmark {
    pub fn new() -> Self {
        GatewayBenchmark {}
    }
}
impl TaskApplicant for GatewayBenchmark {
    fn apply(&self, gateway: &Gateway) -> Result<Vec<Job>, anyhow::Error> {
        Ok(Vec::new())
    }
}