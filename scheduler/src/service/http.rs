use serde_json::json;
use warp::{Rejection, Reply};
use core::models::WorkerInfo;
pub struct HttpService {

}

impl HttpService {
    pub fn builder() -> HttpServiceBuilder {
        HttpServiceBuilder::default()
    }
    pub async fn register_worker(&self, worker_info: WorkerInfo) -> Result<impl Reply, Rejection>{
        print!("Handle register worker request from {:?}", &worker_info);
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
    pub async fn pause_worker(&self, worker_info: WorkerInfo) -> Result<impl Reply, Rejection>{
        print!("Handle register worker request from {:?}", &worker_info);
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
    pub async fn resume_worker(&self, worker_info: WorkerInfo) -> Result<impl Reply, Rejection>{
        print!("Handle register worker request from {:?}", &worker_info);
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
    pub async fn stop_worker(&self, worker_info: WorkerInfo) -> Result<impl Reply, Rejection>{
        print!("Handle register worker request from {:?}", &worker_info);
        return Ok(warp::reply::json(&json!({ "error": "Not implemented" })));
    }
}
pub struct HttpServiceBuilder {
    inner: HttpService,
}

impl Default for HttpServiceBuilder {
    fn default() -> Self {
        Self {
            inner: HttpService {}
        }
    }
}

impl HttpServiceBuilder {
    pub fn build(self) -> HttpService {
        self.inner
    }
}