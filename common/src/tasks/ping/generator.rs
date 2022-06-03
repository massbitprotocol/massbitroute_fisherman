use crate::job_manage::{Job, JobDetail, JobPing};
use crate::tasks::generator::TaskApplicant;
use crate::tasks::LoadConfig;
use crate::{ComponentInfo, Timestamp};
use anyhow::Error;
use async_trait::async_trait;
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
    pub fn new(config_dir: &str) -> Self {
        PingGenerator {
            config: PingConfig::load_config(format!("{}/ping.json", config_dir).as_str()),
        }
    }
    pub fn get_url(&self, component: &ComponentInfo) -> String {
        format!("https://{}/_ping", component.ip)
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

impl TaskApplicant for PingGenerator {
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
