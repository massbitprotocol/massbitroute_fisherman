use common::job_manage::Job;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct JobBuffer {
    jobs: Vec<Job>,
}

impl JobBuffer {
    pub fn new() -> Self {
        JobBuffer { jobs: vec![] }
    }
    pub fn add_job(&mut self, job: Job) {
        self.jobs.push(job);
    }
    pub fn add_jobs(&mut self, mut jobs: Vec<Job>) {
        self.jobs.append(&mut jobs);
    }
    pub fn pop_job(&mut self) -> Option<Job> {
        self.jobs.pop()
    }
}
