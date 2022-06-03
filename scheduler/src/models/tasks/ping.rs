use crate::models::tasks::{LoadConfig, TaskApplicant};
use crate::PING_CONFIG_PATH;
use anyhow::Error;
use common::component::ComponentInfo;
use common::job_manage::{Job, JobDetail, JobPing, JobResult};
use common::Timestamp;
use common::{Gateway, Node};
use log::error;
use minifier::js::Keyword::Default;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::vec;

/*
 * Periodically ping to node/gateway to get response time, to make sure node/gateway is working
 */
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct TaskPing {
    assigned_at: u64, //Timestamp to assign job
    finished_at: u64, //Timestamp when result has arrived
    config: PingConfig,
}

impl TaskPing {
    pub fn new() -> Self {
        TaskPing {
            assigned_at: 0,
            finished_at: 0,
            config: PingConfig::load_config(&*PING_CONFIG_PATH),
        }
    }
    pub fn get_url(&self, component: &ComponentInfo) -> String {
        format!("https://{}/_ping", component.ip)
    }
}

impl TaskApplicant for TaskPing {
    fn apply(&self, component: &ComponentInfo) -> Result<Vec<Job>, Error> {
        log::debug!("TaskPing apply for component {:?}", component);
        let job_ping = JobPing {};
        let mut job = Job::new(JobDetail::Ping(job_ping));
        job.component_url = self.get_url(component);
        job.time_out = self.config.ping_timeout_ms;
        job.repeat_number = self.config.ping_sample_number;
        let vec = vec![job];
        Ok(vec)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
struct PingConfig {
    ping_success_ratio_threshold: f32,
    ping_sample_number: u32,
    ping_request_response: String,
    ping_timeout_ms: Timestamp,
}

impl LoadConfig<PingConfig> for PingConfig {}
