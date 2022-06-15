use crate::component::{ChainInfo, ComponentInfo, ComponentType};
use crate::job_manage::{Config, Job, JobDetail, JobPing, JobResult, JobRole};
use crate::models::PlanEntity;
use crate::tasks::eth::{JobLatestBlock, JobLatestBlockResult};
use crate::tasks::generator::TaskApplicant;
use crate::tasks::LoadConfig;
use crate::util::get_current_time;
use crate::{Node, PlanId, Timestamp, DOMAIN};
use anyhow::Error;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::vec;
/*
 * Apply for node to get latest block number and time
 */
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
struct LatestBlockConfig {
    #[serde(default)]
    header: HashMap<String, String>,
    #[serde(default)]
    latest_block_request_body: String,
    #[serde(default)]
    latest_block_timeout_ms: Timestamp,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct LatestBlockGenerator {
    config: LatestBlockConfig,
}

impl LatestBlockGenerator {
    pub fn new(config_dir: &str, role: &JobRole) -> Self {
        LatestBlockGenerator {
            config: LatestBlockConfig::load_config(
                format!("{}/latest_block.json", config_dir).as_str(),
                role,
            ),
        }
    }
    pub fn get_url(&self, component: &ComponentInfo) -> String {
        format!("https://{}", component.ip)
    }
}

impl LoadConfig<LatestBlockConfig> for LatestBlockConfig {}

impl TaskApplicant for LatestBlockGenerator {
    fn can_apply(&self, component: &ComponentInfo) -> bool {
        return component.component_type == ComponentType::Node;
    }

    fn apply(&self, plan_id: &PlanId, node: &Node) -> Result<Vec<Job>, Error> {
        let job = JobLatestBlock {
            assigned_at: get_current_time(),
            request_body: self.config.latest_block_request_body.clone(),
            chain_info: ChainInfo::new(node.blockchain.clone(), node.network.clone()),
        };
        let job_detail = JobDetail::LatestBlock(job);
        let mut job = Job::new(plan_id.clone(), job_detail.get_job_name(), node, job_detail);
        job.parallelable = true;
        job.timeout = self.config.latest_block_timeout_ms;
        job.component_url = self.get_url(node);
        job.header = self.config.header.clone();
        job.header
            .insert("X-Api-Key".to_string(), node.token.clone());
        job.header.insert(
            "Host".to_string(),
            format!("{}.node.mbr.{}", node.id.clone(), *DOMAIN),
        );
        info!("job header: {:?}", job.header);
        let vec = vec![job];
        Ok(vec)
    }
}
