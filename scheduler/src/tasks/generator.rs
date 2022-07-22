/*
 * Each Task description can apply to node/gateway to generate a list of jobs.
 * If task is not suitable then result is empty
 */
use crate::models::job_result_cache::{PlanTaskResultKey, TaskKey};
use crate::models::jobs::JobAssignmentBuffer;
use crate::models::TaskDependency;
use crate::persistence::PlanModel;
use crate::service::judgment::JudgmentsResult;
use crate::tasks::benchmark::generator::BenchmarkGenerator;
use crate::tasks::websocket::generator::WebsocketGenerator;
use crate::tasks::*;
use crate::CONFIG;
use common::component::ComponentInfo;
use common::job_manage::JobRole;
use common::jobs::{Job, JobAssignment};
use common::tasks::TaskConfigTrait;
use common::util::get_current_time;
use common::workers::{MatchedWorkers, Worker};
use common::{PlanId, Timestamp};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

pub trait TaskApplicant: Sync + Send {
    fn get_type(&self) -> String;
    // fn get_task_names(&self) -> Vec<String> {
    //     Vec::default()
    // }
    // fn has_all_dependent_results(
    //     &self,
    //     _plan_id: &PlanId,
    //     _results: &HashMap<TaskKey, JudgmentsResult>,
    // ) -> bool {
    //     true
    // }
    /*
     * return sub_task
     */
    fn get_task_dependencies(&self, sub_task: &String) -> Vec<TaskKey> {
        Vec::default()
    }
    /*
     * Get all suitable sub task for input component
     */
    fn match_sub_task(&self, component: &ComponentInfo, phase: &JobRole) -> Vec<String>;
    /*

    */
    fn apply(
        &self,
        plan: &PlanId,
        component: &ComponentInfo,
        phase: JobRole,
        workers: &MatchedWorkers,
        sub_tasks: &Vec<String>,
    ) -> Result<JobAssignmentBuffer, anyhow::Error>;
    fn apply_with_cache(
        &self,
        plan: &PlanId,
        component: &ComponentInfo,
        phase: JobRole,
        workers: &MatchedWorkers,
        latest_update: HashMap<String, Timestamp>,
    ) -> Result<JobAssignmentBuffer, anyhow::Error> {
        let task_name = self.get_type();
        let timestamp = latest_update
            .get(&task_name)
            .map(|val| val.clone())
            .unwrap_or_default();
        if get_current_time() - timestamp > CONFIG.generate_new_regular_timeout * 1000 {
            let matched_sub_tasks = self.match_sub_task(component, &phase);
            self.apply(plan, component, phase, workers, &matched_sub_tasks)
        } else {
            Ok(JobAssignmentBuffer::default())
        }
    }
    fn assign_jobs(
        &self,
        _plan: &PlanModel,
        _provider_node: &ComponentInfo,
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
    task_types: &Vec<String>,
) -> Vec<Arc<dyn TaskApplicant>> {
    let mut result: Vec<Arc<dyn TaskApplicant>> = Default::default();
    //Generic http request task
    if task_types.contains(&HttpRequestGenerator::get_name()) {
        result.push(Arc::new(HttpRequestGenerator::new(config_dir, &role)));
    }
    if task_types.contains(&WebsocketGenerator::get_name()) {
        result.push(Arc::new(WebsocketGenerator::new(config_dir, &role)));
    }
    if task_types.contains(&BenchmarkGenerator::get_name()) {
        result.push(Arc::new(BenchmarkGenerator::new(config_dir, &role)));
    }
    // if task_types.contains(&PingGenerator::get_name()) {
    //     result.push(Arc::new(PingGenerator::new(config_dir, &role)));
    // }
    // if task_types.contains(&LatestBlockGenerator::get_name()) {
    //     result.push(Arc::new(LatestBlockGenerator::new(config_dir, &role)));
    // }
    // if task_types.contains(&TaskGWNodeConnection::get_name()) {
    //     result.push(Arc::new(TaskGWNodeConnection::new()));
    // }
    result
}
