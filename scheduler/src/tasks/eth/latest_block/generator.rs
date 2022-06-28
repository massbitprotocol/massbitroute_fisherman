use crate::models::jobs::AssignmentBuffer;
use crate::persistence::PlanModel;
use crate::tasks::generator::TaskApplicant;
use anyhow::Error;
use common::component::{ChainInfo, ComponentInfo, ComponentType};
use common::job_manage::{JobDetail, JobRole};
use common::jobs::{Job, JobAssignment};
use common::tasks::eth::{JobLatestBlock, LatestBlockConfig};
use common::tasks::LoadConfig;
use common::util::get_current_time;
use common::workers::MatchedWorkers;
use common::{Node, PlanId, Timestamp, DOMAIN};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::vec;
/*
 * Apply for node to get latest block number and time
 */

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct LatestBlockGenerator {
    config: LatestBlockConfig,
}

impl LatestBlockGenerator {
    pub fn get_name() -> String {
        String::from("LatestBlock")
    }
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

impl TaskApplicant for LatestBlockGenerator {
    fn get_name(&self) -> String {
        Self::get_name()
    }

    fn can_apply(&self, component: &ComponentInfo) -> bool {
        return component.component_type == ComponentType::Node;
    }

    fn apply(
        &self,
        plan_id: &PlanId,
        node: &Node,
        phase: JobRole,
        workers: &MatchedWorkers,
    ) -> Result<AssignmentBuffer, Error> {
        let job = JobLatestBlock {
            assigned_at: get_current_time(),
            request_body: self.config.latest_block_request_body.clone(),
            chain_info: ChainInfo::new(node.blockchain.clone(), node.network.clone()),
        };
        let job_detail = JobDetail::LatestBlock(job);
        let mut job = Job::new(
            plan_id.clone(),
            job_detail.get_job_name(),
            node,
            job_detail,
            phase,
        );
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
        job.interval = self.config.interval;
        job.repeat_number = self.config.repeat_number as i32;
        debug!("job header: {:?}", job.header);
        let mut assignment_buffer = AssignmentBuffer::default();
        assignment_buffer.assign_job(job, workers);
        Ok(assignment_buffer)
    }
    /*
     * Todo: for testing send jobs to all worker, remote or update this function
     */
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
