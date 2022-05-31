use common::job_manage::JobResult;
use common::{JobId, WorkerId};
use serde::{Deserialize, Serialize};

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
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct JobAssignment {
    worker_id: WorkerId,
    job_id: JobId,
    status: JobStatus,
    assigned_at: u64, //Timestamp to assign job
    finished_at: u64, //Timestamp when result has arrived
    result: Option<JobResult>,
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
