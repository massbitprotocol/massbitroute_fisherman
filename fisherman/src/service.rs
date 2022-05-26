use crate::state::FishermanState;
use common::job_manage::Job;
use common::models::WorkerInfo;
use log::info;
use serde_json::json;
use std::sync::{Arc, Mutex};
use warp::{Rejection, Reply};

pub struct FishermanService {}

impl FishermanService {
    pub fn builder() -> FishermanServiceBuilder {
        FishermanServiceBuilder::default()
    }
    pub async fn handle_jobs(
        &self,
        jobs: Vec<Job>,
        state: Arc<Mutex<FishermanState>>,
    ) -> Result<impl Reply, Rejection> {
        info!("Handle jobs {:?}", &jobs);
        let mut jds = vec![];
        for job in jobs {
            let jd = job.job_detail.unwrap();
            jds.push(jd);
        }
        //serde_json::to_string(jds);
        let res = warp::reply::json(&jds);
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

impl Default for FishermanServiceBuilder {
    fn default() -> Self {
        Self {
            inner: FishermanService {},
        }
    }
}

impl FishermanServiceBuilder {
    pub fn build(self) -> FishermanService {
        self.inner
    }
}
