use crate::models::tasks::eth::{
    GatewayBenchmark, NodeBenchmark, TaskGWNodeConnection, TaskLatestBlock,
};
use crate::models::tasks::ping::TaskPing;
use common::component::ComponentInfo;
use common::job_manage::Job;
use log::debug;
use std::default;
use std::sync::Arc;

pub mod dot;
pub mod eth;
pub mod ping;
/*
 * Each Task description can apply to node/gateway to generate a list of jobs.
 * If task is suitable for node or gateway only then result is empty
 */
pub trait TaskApplicant: Sync + Send {
    fn apply(&self, component: &ComponentInfo) -> Result<Vec<Job>, anyhow::Error>;
}

pub fn get_eth_tasks() -> Vec<Arc<dyn TaskApplicant>> {
    let mut result: Vec<Arc<dyn TaskApplicant>> = Default::default();
    result.push(Arc::new(GatewayBenchmark::new()));
    result.push(Arc::new(NodeBenchmark::new()));
    result.push(Arc::new(TaskGWNodeConnection::new()));
    result.push(Arc::new(TaskLatestBlock::new()));
    result.push(Arc::new(TaskPing::new()));
    result
}

pub fn get_dot_tasks() -> Vec<Arc<dyn TaskApplicant>> {
    let mut result: Vec<Arc<dyn TaskApplicant>> = Default::default();

    result
}
