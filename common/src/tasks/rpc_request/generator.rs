use crate::tasks::generator::TaskApplicant;
use crate::tasks::LoadConfig;
use crate::{ComponentInfo, Timestamp};
use anyhow::Error;
use async_trait::async_trait;

use crate::job_manage::{Job, JobDetail, JobRole};
use crate::models::PlanEntity;
use crate::tasks::rpc_request::JobRpcRequest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::vec;
use tokio::sync::mpsc::Sender;

/*
 * Periodically ping to node/gateway to get response time, to make sure node/gateway is working
 */
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct RpcRequestGenerator {
    config: RpcRequestConfig,
}

impl RpcRequestGenerator {
    pub fn new(config_dir: &str, role: &JobRole) -> Self {
        RpcRequestGenerator {
            config: RpcRequestConfig::load_config(
                format!("{}/rpcrequest.json", config_dir).as_str(),
                role,
            ),
        }
    }
    pub fn get_url(&self, component: &ComponentInfo) -> String {
        format!("https://{}/_ping", component.ip)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
struct RpcRequestConfig {
    #[serde(default)]
    ping_success_ratio_threshold: f32,
    #[serde(default)]
    ping_sample_number: i32,
    #[serde(default)]
    ping_request_response: String,
    #[serde(default)]
    ping_timeout_ms: Timestamp,
}

impl LoadConfig<RpcRequestConfig> for RpcRequestConfig {}

impl TaskApplicant for RpcRequestGenerator {
    fn can_apply(&self, component: &ComponentInfo) -> bool {
        true
    }

    fn apply(&self, plan: &PlanEntity, component: &ComponentInfo) -> Result<Vec<Job>, Error> {
        log::debug!("TaskPing apply for component {:?}", component);
        let detail = JobRpcRequest {};
        let comp_url = detail.get_component_url(component);
        let mut job = Job::new(
            plan.plan_id.clone(),
            String::new(),
            component,
            JobDetail::RpcRequest(detail),
        );
        job.parallelable = true;
        job.component_url = comp_url;
        job.timeout = self.config.ping_timeout_ms;
        job.repeat_number = self.config.ping_sample_number;
        let vec = vec![job];
        Ok(vec)
    }
}
