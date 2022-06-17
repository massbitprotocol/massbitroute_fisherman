use crate::component::{ComponentType, Zone};
use crate::job_manage::{JobDetail, JobResult};
use crate::workers::{Worker, WorkerInfo};
use crate::{
    ComponentId, ComponentInfo, JobId, Timestamp, Url, DEFAULT_JOB_INTERVAL, DEFAULT_JOB_TIMEOUT,
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
    pub fn new(
        plan_id: String,
        job_name: String,
        component: &ComponentInfo,
        job_detail: JobDetail,
    ) -> Self {
        let uuid = Uuid::new_v4();
        Job {
            job_id: uuid.to_string(),
            job_name,
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

#[derive(Clone, Serialize, Deserialize, Debug)]
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
