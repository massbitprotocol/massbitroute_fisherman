use crate::models::job::JobBuffer;
use crate::state::WorkerState;
use common::job_manage::{Job, JobBenchmark};
use common::worker::WorkerInfo;
use common::JobId;
use common::{Deserialize, Serialize};
use log::info;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Rejection, Reply};

pub struct WebService {}

impl WebService {
    pub fn builder(job_buffer: Arc<Mutex<JobBuffer>>) -> WebServiceBuilder {
        WebServiceBuilder {
            inner: WebService {},
        }
    }
    pub async fn handle_jobs(
        &self,
        jobs: Vec<Job>,
        state: Arc<Mutex<WorkerState>>,
    ) -> Result<impl Reply, Rejection> {
        info!("Handle jobs {:?}", &jobs);
        let mut job_ids = vec![];
        for job in jobs.iter() {
            // Create list id for return
            let job_id = job.job_id.clone();
            job_ids.push(job_id);

            // push job into queue
            // {
            //     info!("Push jobs {:?} in to queue", &job.job_id);
            //     let mut lock_state = self.job_queue.lock().unwrap();
            //     lock_state.job_queue.send(job).await;
            //     info!("Jobs queue: {:?}", lock_state.);
            // }
            //self.job_queue.send(job).await;
        }
        state.lock().await.push_jobs(jobs);
        //serde_json::to_string(jds);
        let res = JobHandleResponse::new(job_ids);
        let res = warp::reply::json(&res);
        return Ok(res);
    }
    pub async fn update_jobs(
        &self,
        jobs: Vec<Job>,
        state: Arc<Mutex<WorkerState>>,
    ) -> Result<impl Reply, Rejection> {
        info!("Update jobs: {:?}", &jobs);
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
    pub async fn get_state(&self, state: Arc<Mutex<WorkerState>>) -> Result<impl Reply, Rejection> {
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
