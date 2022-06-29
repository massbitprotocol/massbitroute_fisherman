use crate::jobs::Job;
use serde::{Deserialize, Serialize};

pub mod executor;
pub mod generator;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct JobCommand {}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobCommandResponse {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobCommandResult {
    pub job: Job,
    pub response: JobCommandResponse,
}

impl JobCommandResult {
    pub fn new(job: Job, response: JobCommandResponse) -> Self {
        JobCommandResult { job, response }
    }
}
