use crate::state::{ProcessorState, SchedulerState};
use common::models::WorkerInfo;
use serde_json::json;
use std::sync::{Arc, Mutex};
use warp::{Rejection, Reply};

pub struct SchedulerService {}

impl SchedulerService {
    pub fn builder() -> SchedulerServiceBuilder {
        SchedulerServiceBuilder::default()
    }
    pub async fn register_worker(
        &self,
        worker_info: WorkerInfo,
        scheduler_state: Arc<Mutex<SchedulerState>>,
    ) -> Result<impl Reply, Rejection> {
        print!("Handle register worker request from {:?}", &worker_info);
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
    pub async fn pause_worker(
        &self,
        worker_info: WorkerInfo,
        scheduler_state: Arc<Mutex<SchedulerState>>,
    ) -> Result<impl Reply, Rejection> {
        print!("Handle register worker request from {:?}", &worker_info);
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
    pub async fn resume_worker(
        &self,
        worker_info: WorkerInfo,
        scheduler_state: Arc<Mutex<SchedulerState>>,
    ) -> Result<impl Reply, Rejection> {
        print!("Handle register worker request from {:?}", &worker_info);
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
    pub async fn stop_worker(&self, worker_info: WorkerInfo) -> Result<impl Reply, Rejection> {
        print!("Handle register worker request from {:?}", &worker_info);
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
    //For handler job report
    pub async fn report_result(
        &self,
        job_result: JobResult,
        state: Arc<Mutex<ProcessorState>>,
    ) -> Result<impl Reply, Rejection> {
        print!("Handle report from worker {:?}", &worker_info);
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
}
pub struct SchedulerServiceBuilder {
    inner: SchedulerService,
}

impl Default for SchedulerServiceBuilder {
    fn default() -> Self {
        Self {
            inner: SchedulerService {},
        }
    }
}

impl SchedulerServiceBuilder {
    pub fn build(self) -> SchedulerService {
        self.inner
    }
}
