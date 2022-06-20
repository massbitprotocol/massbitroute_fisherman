/*
 * Each Task description can apply to node/gateway to generate a list of jobs.
 * If task is suitable for node or gateway only then result is empty
 */
use crate::persistence::PlanModel;
use crate::tasks::*;
use common::component::ComponentInfo;
use common::job_manage::JobRole;
use common::jobs::{Job, JobAssignment};
use common::models::PlanEntity;
use common::tasks::eth::*;
use common::workers::{MatchedWorkers, Worker};
use common::PlanId;
use std::sync::Arc;

pub trait TaskApplicant: Sync + Send {
    fn can_apply(&self, component: &ComponentInfo) -> bool;
    fn apply(&self, plan: &PlanId, component: &ComponentInfo) -> Result<Vec<Job>, anyhow::Error>;
    fn assign_jobs(
        &self,
        plan: &PlanModel,
        provider_node: &ComponentInfo,
        jobs: &Vec<Job>,
        workers: &MatchedWorkers,
    ) -> Result<Vec<JobAssignment>, anyhow::Error> {
        let mut assignments = Vec::default();
        let worker_count = workers.nearby_workers.len();
        if worker_count > 0 {
            jobs.iter().enumerate().for_each(|(ind, job)| {
                let wind = ind % worker_count;
                let worker: &Arc<Worker> = workers.nearby_workers.get(wind).unwrap();
                let job_assignment = JobAssignment::new(worker.clone(), job);
                assignments.push(job_assignment);
            });
        }
        Ok(assignments)
    }
}
/*
 * Todo: can add config to load required task for each phase: verification or regular
 */

pub fn get_tasks(
    config_dir: &str,
    role: JobRole,
    task_names: &Vec<String>,
) -> Vec<Arc<dyn TaskApplicant>> {
    let mut result: Vec<Arc<dyn TaskApplicant>> = Default::default();
    if task_names.contains(&HttpRequestGenerator::get_name()) {
        if let Ok(http_request) = HttpRequestGenerator::new(config_dir) {
            result.push(Arc::new(http_request));
        }
    }
    if task_names.contains(&BenchmarkGenerator::get_name()) {
        result.push(Arc::new(BenchmarkGenerator::new(config_dir, &role)));
    }
    if task_names.contains(&PingGenerator::get_name()) {
        result.push(Arc::new(PingGenerator::new(config_dir, &role)));
    }
    if task_names.contains(&LatestBlockGenerator::get_name()) {
        result.push(Arc::new(LatestBlockGenerator::new(config_dir, &role)));
    }
    if task_names.contains(&TaskGWNodeConnection::get_name()) {
        result.push(Arc::new(TaskGWNodeConnection::new()));
    }
    result
}
