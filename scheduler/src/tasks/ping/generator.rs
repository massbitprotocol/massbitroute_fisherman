use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobDetail, JobPing, JobRole};
use common::tasks::LoadConfig;
use std::str::FromStr;

use crate::models::jobs::AssignmentBuffer;
use crate::persistence::PlanModel;
use crate::tasks::generator::TaskApplicant;
use common::component::ComponentInfo;
use common::jobs::{Job, JobAssignment};
use common::models::PlanEntity;
use common::workers::{MatchedWorkers, Worker};
use common::{PlanId, Timestamp};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::vec;
use tokio::sync::mpsc::Sender;

/*
 * Periodically ping to node/gateway to get response time, to make sure node/gateway is working
 */
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct PingGenerator {
    config: PingConfig,
}

impl PingGenerator {
    pub fn get_name() -> String {
        String::from("Ping")
    }
    pub fn new(config_dir: &str, role: &JobRole) -> Self {
        PingGenerator {
            config: PingConfig::load_config(format!("{}/ping.json", config_dir).as_str(), role),
        }
    }
    pub fn get_url(&self, component: &ComponentInfo) -> String {
        format!("https://{}/_ping", component.ip)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct PingConfig {
    pub ping_success_percent_threshold: f64, //
    pub ping_percentile: f64,
    pub ping_response_time_threshold: u64,
    pub repeat_number: i32,
    pub ping_request_response: String,
    pub ping_timeout_ms: Timestamp,
    pub ping_number_for_decide: i32,
}

impl LoadConfig<PingConfig> for PingConfig {}

impl TaskApplicant for PingGenerator {
    fn get_name(&self) -> String {
        String::from("Ping")
    }
    fn can_apply(&self, component: &ComponentInfo) -> bool {
        true
    }

    fn apply(
        &self,
        plan_id: &PlanId,
        component: &ComponentInfo,
        phase: JobRole,
        workers: &MatchedWorkers,
    ) -> Result<AssignmentBuffer, Error> {
        log::debug!("TaskPing apply for component {:?}", component);
        let job_ping = JobPing {};
        let job_detail = JobDetail::Ping(job_ping);
        let mut job = Job::new(
            plan_id.clone(),
            job_detail.get_job_name(),
            component,
            job_detail,
            phase,
        );
        job.parallelable = true;
        job.component_url = self.get_url(component);
        job.timeout = self.config.ping_timeout_ms;
        job.repeat_number = self.config.repeat_number;
        let mut assignment_buffer = AssignmentBuffer::default();
        assignment_buffer.assign_job(job, workers);
        Ok(assignment_buffer)
    }
    fn assign_jobs(
        &self,
        plan: &PlanModel,
        provider_node: &ComponentInfo,
        jobs: &Vec<Job>,
        workers: &MatchedWorkers,
    ) -> Result<Vec<JobAssignment>, anyhow::Error> {
        let phase = JobRole::from_str(plan.phase.as_str())?;
        let mut assignments = Vec::default();
        match phase {
            JobRole::Verification => {
                jobs.iter().enumerate().for_each(|(ind, job)| {
                    for worker in workers.best_workers.iter() {
                        let job_assignment = JobAssignment::new(worker.clone(), job);
                        assignments.push(job_assignment);
                    }
                });
            }
            JobRole::Regular => {
                let worker_count = workers.nearby_workers.len();
                if worker_count > 0 {
                    jobs.iter().enumerate().for_each(|(ind, job)| {
                        let wind = ind % worker_count;
                        let worker: &Arc<Worker> = workers.nearby_workers.get(wind).unwrap();
                        let job_assignment = JobAssignment::new(worker.clone(), job);
                        assignments.push(job_assignment);
                    });
                }
            }
        }
        Ok(assignments)
    }
}
