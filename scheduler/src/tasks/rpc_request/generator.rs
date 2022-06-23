use anyhow::Error;
use async_trait::async_trait;
use common::tasks::LoadConfig;

use crate::tasks::generator::TaskApplicant;
use common::component::ComponentInfo;
use common::job_manage::{JobDetail, JobRole};
use common::jobs::Job;
use common::models::PlanEntity;
use common::tasks::rpc_request::JobRpcRequest;
use common::{PlanId, Timestamp};
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
    fn get_name(&self) -> String {
        String::from("RpcRequest")
    }

    fn can_apply(&self, component: &ComponentInfo) -> bool {
        true
    }

    fn apply(&self, plan_id: &PlanId, component: &ComponentInfo) -> Result<Vec<Job>, Error> {
        log::debug!("TaskPing apply for component {:?}", component);
        let detail = JobRpcRequest {};
        let comp_url = detail.get_component_url(component);
        let mut job = Job::new(plan_id.clone(), component, JobDetail::RpcRequest(detail));
        job.parallelable = true;
        job.component_url = comp_url;
        job.timeout = self.config.ping_timeout_ms;
        job.repeat_number = self.config.ping_sample_number;
        let vec = vec![job];
        Ok(vec)
    }
}
