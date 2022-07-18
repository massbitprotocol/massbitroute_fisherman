use crate::models::jobs::JobAssignmentBuffer;
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
use common::{Node, PlanId, DOMAIN};
use log::debug;
use serde::{Deserialize, Serialize};
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
    ) -> Result<JobAssignmentBuffer, Error> {
        let job = JobLatestBlock {
            assigned_at: get_current_time(),
            request_body: self.config.latest_block_request_body.clone(),
            chain_info: ChainInfo::new(node.blockchain.clone(), node.network.clone()),
        };
        let job_detail = JobDetail::LatestBlock(job);
        let mut job = Job::new(
            plan_id.clone(),
            Self::get_name(),
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
        let mut assignment_buffer = JobAssignmentBuffer::default();
        assignment_buffer.assign_job(job, workers, &self.config.assignment);
        Ok(assignment_buffer)
    }
    /*
     * Todo: for testing send jobs to all worker, remote or update this function
     */
    fn assign_jobs(
        &self,
        _plan: &PlanModel,
        _provider_node: &ComponentInfo,
        jobs: &Vec<Job>,
        workers: &MatchedWorkers,
    ) -> Result<Vec<JobAssignment>, anyhow::Error> {
        let mut assignments = Vec::default();
        jobs.iter().enumerate().for_each(|(_ind, job)| {
            for worker in workers.measured_workers.iter() {
                let job_assignment = JobAssignment::new(worker.clone(), job);
                assignments.push(job_assignment);
            }
        });
        Ok(assignments)
    }
}
