use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobDetail, JobPing, JobRole};
use common::tasks::LoadConfig;

use crate::models::tasks::generator::TaskApplicant;
use crate::persistence::PlanModel;
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
    #[serde(default)]
    pub ping_error_percent_threshold: f64, //
    #[serde(default)]
    pub ping_percentile: f64,
    #[serde(default)]
    pub ping_response_time_threshold: u64,
    #[serde(default)]
    pub ping_sample_number: i32,
    #[serde(default)]
    pub ping_request_response: String,
    #[serde(default)]
    pub ping_timeout_ms: Timestamp,
}

impl LoadConfig<PingConfig> for PingConfig {}

impl TaskApplicant for PingGenerator {
    fn can_apply(&self, component: &ComponentInfo) -> bool {
        true
    }

    fn apply(&self, plan_id: &PlanId, component: &ComponentInfo) -> Result<Vec<Job>, Error> {
        log::debug!("TaskPing apply for component {:?}", component);
        let job_ping = JobPing {};
        let job_detail = JobDetail::Ping(job_ping);
        let mut job = Job::new(plan_id.clone(), component, job_detail);
        job.parallelable = true;
        job.component_url = self.get_url(component);
        job.timeout = self.config.ping_timeout_ms;
        job.repeat_number = self.config.ping_sample_number;
        let vec = vec![job];
        Ok(vec)
    }
    fn assign_jobs(
        &self,
        plan: &PlanModel,
        provider_node: &ComponentInfo,
        jobs: &Vec<Job>,
        workers: &MatchedWorkers,
    ) -> Result<Vec<JobAssignment>, anyhow::Error> {
        let mut assignments = Vec::default();
        jobs.iter().enumerate().for_each(|(ind, job)| {
            for worker in workers.best_workers.iter() {
                let job_assignment = JobAssignment::new(worker.clone(), job);
                assignments.push(job_assignment);
            }
        });
        Ok(assignments)
    }
}
