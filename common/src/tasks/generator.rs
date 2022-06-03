/*
 * Each Task description can apply to node/gateway to generate a list of jobs.
 * If task is suitable for node or gateway only then result is empty
 */
use crate::job_manage::Job;
use crate::tasks::eth::*;
use crate::tasks::ping::generator::PingGenerator;
use crate::ComponentInfo;
use std::sync::Arc;

pub trait TaskApplicant: Sync + Send {
    fn apply(&self, component: &ComponentInfo) -> Result<Vec<Job>, anyhow::Error>;
}

pub fn get_eth_task_genrators(config_dir: &str) -> Vec<Arc<dyn TaskApplicant>> {
    let mut result: Vec<Arc<dyn TaskApplicant>> = Default::default();
    result.push(Arc::new(GatewayBenchmark::new()));
    result.push(Arc::new(NodeBenchmark::new()));
    result.push(Arc::new(TaskGWNodeConnection::new()));
    result.push(Arc::new(TaskLatestBlock::new()));
    result.push(Arc::new(PingGenerator::new(config_dir)));
    result
}

pub fn get_dot_task_generators() -> Vec<Arc<dyn TaskApplicant>> {
    let mut result: Vec<Arc<dyn TaskApplicant>> = Default::default();

    result
}
