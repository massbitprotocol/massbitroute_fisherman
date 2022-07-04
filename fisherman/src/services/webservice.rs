use crate::models::job::JobBuffer;
use crate::state::WorkerState;
use common::jobs::Job;
use common::JobId;
use common::{Deserialize, Serialize};
use log::{info, trace};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Rejection, Reply};

pub struct WebService {}

impl WebService {
    pub fn builder(_job_buffer: Arc<Mutex<JobBuffer>>) -> WebServiceBuilder {
        WebServiceBuilder {
            inner: WebService {},
        }
    }
    pub async fn handle_jobs(
        &self,
        jobs: Vec<Job>,
        state: Arc<Mutex<WorkerState>>,
    ) -> Result<impl Reply, Rejection> {
        trace!("Handle {} jobs {:?}", jobs.len(), &jobs);
        let job_ids = jobs
            .iter()
            .map(|job| {
                // Create list id for return
                job.job_id.clone()
            })
            .collect();
        {
            let mut lock = state.lock().await;
            let size = lock.push_jobs(jobs).await;
            info!("There are {} jobs in queue", size);
        }

        //serde_json::to_string(jds);
        let res = JobHandleResponse::new(job_ids);
        let res = warp::reply::json(&res);
        return Ok(res);
    }
    pub async fn update_jobs(
        &self,
        jobs: Vec<Job>,
        _state: Arc<Mutex<WorkerState>>,
    ) -> Result<impl Reply, Rejection> {
        info!("Update jobs: {:?}", &jobs);
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
    pub async fn get_state(
        &self,
        _state: Arc<Mutex<WorkerState>>,
    ) -> Result<impl Reply, Rejection> {
        info!("Get state request");
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
}
pub struct WebServiceBuilder {
    inner: WebService,
}

impl WebServiceBuilder {
    pub fn new() -> Self {
        WebServiceBuilder {
            inner: WebService {},
        }
    }

    pub fn build(self) -> WebService {
        self.inner
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
struct JobHandleResponse {
    success: bool,
    result: Vec<JobId>,
}

impl JobHandleResponse {
    pub fn new(job_ids: Vec<JobId>) -> Self {
        JobHandleResponse {
            success: true,
            result: job_ids,
        }
    }
}
