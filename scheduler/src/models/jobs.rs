use common::job_manage::JobResult;
use common::{JobId, WorkerId};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct AssignmentBuffer {
    list_assignments: Vec<JobAssignment>,
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobAssignment {
    worker_id: WorkerId,
    job_id: JobId,
    status: JobStatus,
    assigned_at: u64, //Timestamp to assign job
    finished_at: u64, //Timestamp when result has arrived
    result: JobResult,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, Hash, Eq)]
pub enum JobStatus {
    // Initial status, when job is assigned to a worker
    ASSIGNED,
    //Job is send to worker
    DELIVERED,
    //Receive job result
    DONE,
}
