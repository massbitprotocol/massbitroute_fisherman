use crate::models::workers::Worker;
use common::component::Zone;
use common::job_manage::{Job, JobResult};
use common::worker::WorkerInfo;
use common::{JobId, WorkerId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AssignmentBuffer {
    list_assignments: Vec<JobAssignment>,
}
impl Default for AssignmentBuffer {
    fn default() -> Self {
        AssignmentBuffer {
            list_assignments: vec![],
        }
    }
}

impl AssignmentBuffer {
    pub fn add_assignments(&mut self, mut assignments: Vec<JobAssignment>) {
        self.list_assignments.append(&mut assignments);
    }
    pub fn push_back(&mut self, mut assignments: Vec<JobAssignment>) {
        for job in assignments.into_iter().rev() {
            self.list_assignments.insert(0, job);
        }
    }
    pub fn pop_all(&mut self) -> Vec<JobAssignment> {
        let mut res = Vec::default();
        res.append(&mut self.list_assignments);
        res
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct JobAssignment {
    pub zone: Zone,
    pub worker: Arc<WorkerInfo>,
    pub job: Job,
    pub status: JobStatus,
    pub assigned_at: u64, //Timestamp to assign job
    pub finished_at: u64, //Timestamp when result has arrived
    pub result: Option<JobResult>,
}

impl JobAssignment {
    pub fn new(worker: Arc<WorkerInfo>, job: &Job) -> JobAssignment {
        JobAssignment {
            zone: worker.zone.clone(),
            worker,
            job: job.clone(),
            status: Default::default(),
            assigned_at: 0,
            finished_at: 0,
            result: None,
        }
    }
}
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
