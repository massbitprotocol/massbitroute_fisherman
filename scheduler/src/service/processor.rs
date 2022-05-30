use crate::state::ProcessorState;
use common::job_manage::JobResult;
use common::worker::WorkerInfo;
use serde_json::json;
use std::sync::{Arc, Mutex};
use warp::{Rejection, Reply};

#[derive(Default)]
pub struct ProcessorService {}

impl ProcessorService {
    pub fn builder() -> ProcessorServiceBuilder {
        ProcessorServiceBuilder::default()
    }
    pub async fn process_report(
        &self,
        job_result: JobResult,
        state: Arc<Mutex<ProcessorState>>,
    ) -> Result<impl Reply, Rejection> {
        print!("Handle report from worker {:?}", &job_result);
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
}

pub struct ProcessorServiceBuilder {
    inner: ProcessorService,
}

impl Default for ProcessorServiceBuilder {
    fn default() -> Self {
        Self {
            inner: ProcessorService {},
        }
    }
}

impl ProcessorServiceBuilder {
    pub fn build(self) -> ProcessorService {
        self.inner
    }
}
