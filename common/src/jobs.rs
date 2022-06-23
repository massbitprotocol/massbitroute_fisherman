use crate::component::{ChainInfo, ComponentType, Zone};
use crate::job_manage::{JobDetail, JobResultDetail, JobRole};
use crate::util::get_current_time;
use crate::workers::{Worker, WorkerInfo};
use crate::{
    ComponentId, ComponentInfo, JobId, Timestamp, Url, WorkerId, DEFAULT_JOB_INTERVAL,
    DEFAULT_JOB_TIMEOUT,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Job {
    pub job_id: JobId,
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
    pub header: HashMap<String, String>,
    pub job_detail: Option<JobDetail>,
}

impl From<&Job> for reqwest::Body {
    fn from(job: &Job) -> Self {
        reqwest::Body::from(serde_json::to_string(job).unwrap())
    }
}
impl Job {
    pub fn new(plan_id: String, component: &ComponentInfo, job_detail: JobDetail) -> Self {
        let uuid = Uuid::new_v4();
        Job {
            job_id: uuid.to_string(),
            job_name: job_detail.get_job_name(),
            plan_id,
            component_id: component.id.clone(),
            component_type: component.component_type.clone(),
            priority: 1,
            expected_runtime: 0,
            repeat_number: 0,
            timeout: DEFAULT_JOB_TIMEOUT,
            interval: DEFAULT_JOB_INTERVAL,
            header: Default::default(),
            job_detail: Some(job_detail),
            parallelable: false,
            component_url: "".to_string(),
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobResult {
    pub job_id: String,
    pub job_name: String,
    pub worker_id: WorkerId,
    pub provider_id: ComponentId,
    pub provider_type: ComponentType,
    pub phase: JobRole,
    pub result_detail: JobResultDetail,
    pub receive_timestamp: Timestamp, //time the worker received result
    pub chain_info: Option<ChainInfo>,
}

impl JobResult {
    pub fn new(result_detail: JobResultDetail, chain_info: Option<ChainInfo>) -> JobResult {
        let receive_timestamp = get_current_time();
        JobResult {
            job_id: "".to_string(),
            job_name: "".to_string(),
            worker_id: "".to_string(),
            provider_id: "".to_string(),
            provider_type: Default::default(),
            phase: Default::default(),
            result_detail,
            receive_timestamp,
            chain_info,
        }
    }
}
