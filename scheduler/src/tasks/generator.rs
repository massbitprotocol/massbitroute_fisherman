/*
 * Each Task description can apply to node/gateway to generate a list of jobs.
 * If task is suitable for node or gateway only then result is empty
 */
use crate::models::jobs::AssignmentBuffer;
use crate::models::TaskDependency;
use crate::persistence::PlanModel;
use crate::tasks::benchmark::generator::BenchmarkGenerator;
use crate::tasks::*;
use crate::CONFIG;
use common::component::ComponentInfo;
use common::job_manage::JobRole;
use common::jobs::{Job, JobAssignment};
use common::models::PlanEntity;
use common::tasks::eth::*;
use common::util::get_current_time;
use common::workers::{MatchedWorkers, Worker};
use common::{PlanId, Timestamp};
use log::info;
use std::collections::HashMap;
use std::sync::Arc;

pub trait TaskApplicant: Sync + Send {
    fn get_name(&self) -> String;
    fn get_task_dependencies(&self) -> TaskDependency {
        TaskDependency::default()
    }
    fn can_apply(&self, component: &ComponentInfo) -> bool;
    fn apply(
        &self,
        plan: &PlanId,
        component: &ComponentInfo,
        phase: JobRole,
        workers: &MatchedWorkers,
    ) -> Result<AssignmentBuffer, anyhow::Error>;
    fn apply_with_cache(
        &self,
        plan: &PlanId,
        component: &ComponentInfo,
        phase: JobRole,
        workers: &MatchedWorkers,
        latest_update: HashMap<String, Timestamp>,
    ) -> Result<AssignmentBuffer, anyhow::Error> {
        let task_name = self.get_name();
        let timestamp = latest_update
            .get(&task_name)
            .map(|val| val.clone())
            .unwrap_or_default();
        if get_current_time() - timestamp > CONFIG.generate_new_regular_timeout * 1000 {
            self.apply(plan, component, phase, workers)
        } else {
            Ok(AssignmentBuffer::default())
        }
    }
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
    //Generic http request task
    if task_names.contains(&HttpRequestGenerator::get_name()) {
        result.push(Arc::new(HttpRequestGenerator::new(config_dir)));
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
