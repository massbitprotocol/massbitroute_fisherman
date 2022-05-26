use crate::state::FishermanState;
use common::job_manage::Job;
use common::models::WorkerInfo;
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
        print!("Handle jobs {:?}", &jobs);
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
    pub async fn update_jobs(
        &self,
        jobs: Vec<Job>,
        state: Arc<Mutex<FishermanState>>,
    ) -> Result<impl Reply, Rejection> {
        print!("Update jobs: {:?}", &jobs);
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
    pub async fn get_state(&self) -> Result<impl Reply, Rejection> {
        print!("Get state request");
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
