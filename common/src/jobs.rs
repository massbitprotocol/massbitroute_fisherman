use crate::component::{ChainInfo, ComponentType, Zone};
use crate::job_manage::{JobDetail, JobResultDetail, JobRole};
use crate::util::get_current_time;
use crate::workers::Worker;
use crate::{
    ComponentId, ComponentInfo, JobId, PlanId, Timestamp, Url, WorkerId, DEFAULT_JOB_INTERVAL,
    DEFAULT_JOB_TIMEOUT, WORKER_ID,
};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, Hash, Eq)]
pub enum JobStatus {
    CREATED,
    // Initial status, when job is assigned to a worker
    ASSIGNED,
    //Job is send to worker
    DELIVERED,
    //Receive job result
    DONE,
}

impl Default for JobStatus {
    fn default() -> Self {
        JobStatus::CREATED
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Job {
    pub job_id: JobId,
    pub job_type: String,
    pub job_name: String,
    pub plan_id: String,
    pub component_id: ComponentId,     //ProviderId
    pub component_type: ComponentType, //ProviderType
    pub priority: i32,                 //Fist priority is 1
    pub expected_runtime: Timestamp, //timestamp in millisecond Default 0, job is executed if only expected_runtime <= current timestamp
    pub parallelable: bool,          //Job can be executed parallel with other jobs
    pub timeout: Timestamp,
    pub component_url: Url,
    pub repeat_number: i32,  //0-don't repeat
    pub interval: Timestamp, //
    pub job_detail: JobDetail,
    pub phase: JobRole,
}

impl From<&Job> for reqwest::Body {
    fn from(job: &Job) -> Self {
        reqwest::Body::from(serde_json::to_string(job).unwrap())
    }
}

impl Job {
    pub fn is_eq(&self, job: &Job) -> bool {
        self.job_name == job.job_name
            && self.job_type == job.job_type
            && self.job_detail == job.job_detail
            && self.phase == job.phase
            && self.component_id == job.component_id
    }

    pub fn new(
        plan_id: String,
        job_type: String,
        job_name: String,
        component: &ComponentInfo,
        job_detail: JobDetail,
        phase: JobRole,
    ) -> Self {
        let uuid = Uuid::new_v4();
        Job {
            job_id: uuid.to_string(),
            job_type,
            job_name,
            plan_id,
            component_id: component.id.clone(),
            component_type: component.component_type.clone(),
            priority: 1,
            expected_runtime: 0,
            repeat_number: 0,
            timeout: DEFAULT_JOB_TIMEOUT,
            interval: DEFAULT_JOB_INTERVAL,
            job_detail,
            parallelable: false,
            component_url: "".to_string(),
            phase,
        }
    }
}

#[derive(Clone, Debug)]
pub struct JobAssignment {
    pub zone: Zone,
    pub worker: Arc<Worker>,
    pub job: Job,
    pub status: JobStatus,
    pub assigned_at: u64, //Timestamp to assign job
    pub finished_at: u64, //Timestamp when result has arrived
    pub result: Option<JobResult>,
}

impl JobAssignment {
    pub fn new(worker: Arc<Worker>, job: &Job) -> JobAssignment {
        JobAssignment {
            zone: worker.get_zone(),
            worker,
            job: job.clone(),
            status: Default::default(),
            assigned_at: 0,
            finished_at: 0,
            result: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct AssignmentConfig {
    pub broadcast: Option<bool>,
    pub worker_number: Option<usize>,
    pub nearby_only: Option<bool>,
    pub by_distance: Option<bool>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct JobResult {
    pub plan_id: PlanId,
    pub job_id: JobId,
    pub job_name: String, //For http request use task_name in task config ex: RoundTripTime/LatestBlock
    pub worker_id: WorkerId,
    pub provider_id: ComponentId,
    pub provider_type: ComponentType,
    pub phase: JobRole,
    pub result_detail: JobResultDetail,
    pub receive_timestamp: Timestamp, //time the worker received result
    pub chain_info: Option<ChainInfo>,
}

impl Debug for JobResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let result = match &self.result_detail {
            JobResultDetail::HttpRequest(detail) => {
                format!("{:?}", detail.response)
            }
            JobResultDetail::Websocket(detail) => {
                format!(
                    "{:?}, error_code: {}, message: {}",
                    detail.detail, detail.error_code, detail.message
                )
            }
            JobResultDetail::Benchmark(detail) => {
                format!("{:?}", detail.response)
            }
            _ => Default::default(),
        };
        write!(
            f,
            "({}, phase:{:?},result {} , worker: {}, job_id:{})",
            self.job_name, self.phase, result, self.worker_id, self.job_id
        )
    }
}

impl JobResult {
    pub fn new(
        result_detail: JobResultDetail,
        chain_info: Option<ChainInfo>,
        job: &Job,
    ) -> JobResult {
        let receive_timestamp = get_current_time();
        JobResult {
            plan_id: job.plan_id.clone(),
            job_id: job.job_id.clone(),
            job_name: job.job_name.clone(),
            worker_id: WORKER_ID.clone(),
            provider_id: job.component_id.clone(),
            provider_type: job.component_type.clone(),
            phase: job.phase.clone(),
            result_detail,
            receive_timestamp,
            chain_info,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobResultWithJob {
    pub job_result: JobResult,
    pub job: Job,
}

impl JobResultWithJob {
    pub fn new(job_result: JobResult, job: Job) -> Self {
        Self { job_result, job }
    }
}
