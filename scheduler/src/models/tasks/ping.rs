use crate::models::tasks::TaskApplicant;
use anyhow::Error;
use common::component::ComponentInfo;
use common::job_manage::{Job, JobDetail, JobPing, JobResult};
use common::{Gateway, Node};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::vec;
/*
 * Periodically ping to node/gateway to get response time, to make sure node/gateway is working
 */
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct TaskPing {
    assigned_at: u64, //Timestamp to assign job
    finished_at: u64, //Timestamp when result has arrived
}

impl TaskPing {
    pub fn new() -> Self {
        TaskPing {
            assigned_at: 0,
            finished_at: 0,
        }
    }
}

impl TaskApplicant for TaskPing {
    fn apply(&self, component: Arc<ComponentInfo>) -> Result<Vec<Job>, Error> {
        let job_ping = JobPing {};
        let job = Job::new(JobDetail::Ping(job_ping));
        let vec = vec![job];
        Ok(vec)
    }
}
