use common::models::WorkerInfo;
use serde_json::json;
use std::sync::{Arc, Mutex};
use warp::{Rejection, Reply};
pub struct ProcessorService {}

impl ProcessorService {
    pub fn builder() -> ProcessorServiceBuilder {
        ProcessorServiceBuilder::default()
    }
    pub async fn process_report(
        &self,
        worker_info: WorkerInfo,
        state: Arc<Mutex<ReportState>>,
    ) -> Result<impl Reply, Rejection> {
        info!("Handle report from worker {:?}", &worker_info);
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
