use crate::state::SchedulerState;

use common::component::ComponentInfo;
use common::workers::{WorkerInfo, WorkerRegisterResult};

use serde_json::json;
use std::sync::Arc;

use warp::{Rejection, Reply};

#[derive(Default)]
pub struct WebService {}

impl WebService {
    pub fn builder() -> SchedulerServiceBuilder {
        SchedulerServiceBuilder::default()
    }
    pub async fn register_worker(
        &self,
        worker_info: WorkerInfo,
        scheduler_state: Arc<SchedulerState>,
    ) -> Result<impl Reply, Rejection> {
        log::debug!("Handle register worker request from {:?}", &worker_info);
        match scheduler_state.register_worker(worker_info).await {
            Ok(result) => Ok(warp::reply::json(&result)),
            Err(_err) => {
                let result = WorkerRegisterResult {
                    worker_id: "".to_string(),
                    report_callback: "".to_string(),
                };
                Ok(warp::reply::json(&result))
            }
        }
    }
    pub async fn pause_worker(
        &self,
        worker_info: WorkerInfo,
        _scheduler_state: Arc<SchedulerState>,
    ) -> Result<impl Reply, Rejection> {
        print!("Handle register worker request from {:?}", &worker_info);
        Ok(warp::reply::json(&json!({ "error": "Not implemented" })))
    }
    pub async fn resume_worker(
        &self,
        worker_info: WorkerInfo,
        _scheduler_state: Arc<SchedulerState>,
    ) -> Result<impl Reply, Rejection> {
        print!("Handle register worker request from {:?}", &worker_info);
        Ok(warp::reply::json(&json!({ "error": "Not implemented" })))
    }
    pub async fn stop_worker(
        &self,
        worker_info: WorkerInfo,
        _scheduler_state: Arc<SchedulerState>,
    ) -> Result<impl Reply, Rejection> {
        print!("Handle register worker request from {:?}", &worker_info);
        Ok(warp::reply::json(&json!({ "error": "Not implemented" })))
    }
    pub async fn node_verify(
        &self,
        node_info: ComponentInfo,
        scheduler_state: Arc<SchedulerState>,
    ) -> impl Reply {
        log::info!("Handle node verify request from {:?}", &node_info);
        let res = scheduler_state.verify_node(node_info).await;
        if res.is_err() {
            return warp::reply::json(&json!({ "success": false }));
        }
        warp::reply::json(&json!({ "success": true }))
    }
}
pub struct SchedulerServiceBuilder {
    inner: WebService,
}

impl Default for SchedulerServiceBuilder {
    fn default() -> Self {
        Self {
            inner: WebService {},
        }
    }
}
impl SchedulerServiceBuilder {
    pub fn build(self) -> WebService {
        self.inner
    }
}
