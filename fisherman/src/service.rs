use crate::state::FishermanState;
use common::job_manage::Job;
use common::models::WorkerInfo;
use common::JobId;
use common::{Deserialize, Serialize};
use log::info;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;
use warp::{Rejection, Reply};

pub struct FishermanService {
    pub job_queue: Sender<Job>,
}

impl FishermanService {
    pub fn builder(job_queue: Sender<Job>) -> FishermanServiceBuilder {
        FishermanServiceBuilder {
            inner: FishermanService { job_queue },
        }
    }
    pub async fn handle_jobs(
        &self,
        jobs: Vec<Job>,
        state: Arc<Mutex<FishermanState>>,
    ) -> Result<impl Reply, Rejection> {
        info!("Handle jobs {:?}", &jobs);
        let mut job_ids = vec![];
        for job in jobs {
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
            self.job_queue.send(job).await;
        }
        //serde_json::to_string(jds);
        let res = JobHandleResponse::new(job_ids);
        let res = warp::reply::json(&res);
        return Ok(res);
    }
    pub async fn update_jobs(
        &self,
        jobs: Vec<Job>,
        state: Arc<Mutex<FishermanState>>,
    ) -> Result<impl Reply, Rejection> {
        info!("Update jobs: {:?}", &jobs);
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
    pub async fn get_state(&self) -> Result<impl Reply, Rejection> {
        info!("Get state request");
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
}
pub struct FishermanServiceBuilder {
    inner: FishermanService,
}

impl FishermanServiceBuilder {
    pub fn new(sender: Sender<Job>) -> Self {
        FishermanServiceBuilder {
            inner: FishermanService { job_queue: sender },
        }
    }

    pub fn build(self) -> FishermanService {
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
