/*
 * Each Task description can apply to node/gateway to generate a list of jobs.
 * If task is suitable for node or gateway only then result is empty
 */
use crate::job_manage::{Job, JobRole};
use crate::models::PlanEntity;
use crate::tasks::eth::*;
use crate::tasks::http_request::HttpRequestGenerator;
use crate::tasks::ping::generator::PingGenerator;
use crate::ComponentInfo;
use std::sync::Arc;

pub trait TaskApplicant: Sync + Send {
    fn can_apply(&self, component: &ComponentInfo) -> bool;
    fn apply(
        &self,
        plan: &PlanEntity,
        component: &ComponentInfo,
    ) -> Result<Vec<Job>, anyhow::Error>;
}
/*
 * Todo: can add config to load required task for each phase: verification or regular
 */

pub fn get_tasks(config_dir: &str, role: JobRole) -> Vec<Arc<dyn TaskApplicant>> {
    let mut result: Vec<Arc<dyn TaskApplicant>> = Default::default();
    if let Ok(http_request) = HttpRequestGenerator::new(config_dir) {
        result.push(Arc::new(http_request));
    }
    match role {
        JobRole::Verification => {
            result.push(Arc::new(BenchmarkGenerator::new(config_dir, &role)));
        }
        _ => {}
    }
    result.push(Arc::new(TaskGWNodeConnection::new()));
    result.push(Arc::new(TaskLatestBlock::new()));
    result.push(Arc::new(PingGenerator::new(config_dir, &role)));
    result
}
