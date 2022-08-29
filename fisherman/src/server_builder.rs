use crate::server_config::AccessControl;
use common::jobs::Job;
use log::{info, trace};
use serde::{Deserialize, Serialize};

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use warp::http::{HeaderMap, Method};

use crate::services::WebService;
use crate::state::WorkerState;
use crate::BUILD_VERSION;
use common::workers::WorkerStateParam;
use common::{JobId, PlanId};
use serde_json::json;
use std::default::Default;
use tokio::sync::Mutex;
use warp::{http::StatusCode, Filter, Rejection, Reply};

pub const MAX_JSON_BODY_SIZE: u64 = 1024 * 1024;

#[derive(Default)]
pub struct WebServerBuilder {
    entry_point: String,
    access_control: AccessControl,
    worker_state: Arc<Mutex<WorkerState>>,
}

pub struct WorkerServer {
    entry_point: String,
    access_control: AccessControl,
    pub web_service: Arc<WebService>,
    worker_state: Arc<Mutex<WorkerState>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeployParam {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SimpleResponse {
    success: bool,
}

impl WorkerServer {
    pub fn builder() -> WebServerBuilder {
        WebServerBuilder::default()
    }
    pub async fn serve(&self) {
        let allow_headers: Vec<String> = self.access_control.get_access_control_allow_headers();
        info!("allow_headers: {:?}", allow_headers);
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(allow_headers)
            .allow_methods(&[
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
                Method::HEAD,
            ]);
        info!("cors: {:?}", cors);
        let router = self
            .create_ping()
            .with(&cors)
            //.create_get_status(self.scheduler_service.clone()).await.with(&cors)
            .or(self
                .create_route_handle_jobs(self.web_service.clone(), self.worker_state.clone())
                .with(&cors))
            .or(self
                .create_route_cancel_jobs(self.web_service.clone(), self.worker_state.clone())
                .with(&cors))
            .or(self
                .create_route_cancel_plans(self.web_service.clone(), self.worker_state.clone())
                .with(&cors))
            .or(self.create_version().with(&cors))
            .or(self
                .create_route_update_jobs(self.web_service.clone(), self.worker_state.clone())
                .with(&cors))
            .or(self
                .create_route_get_state(self.web_service.clone(), self.worker_state.clone())
                .with(&cors))
            .recover(handle_rejection);
        let socket_addr: SocketAddr = self.entry_point.parse().unwrap();

        warp::serve(router).run(socket_addr).await;
    }

    /// Ping API
    fn create_ping(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("ping")
            .and(warp::get())
            .and_then(move || async move {
                info!("Receive ping request");
                Self::simple_response(true).await
            })
    }

    /// Version API
    fn create_version(&self) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
        warp::path!("version").and(warp::get()).map(move || {
            info!("Receive get version request");
            let res = json!({ "version": &*BUILD_VERSION });
            info!("BUILD_VERSION: {}", &*BUILD_VERSION);
            Ok(warp::reply::json(&res))
        })
    }

    pub(crate) async fn simple_response(success: bool) -> Result<impl Reply, Rejection> {
        let res = SimpleResponse { success };
        Ok(warp::reply::json(&res))
    }
    fn create_route_handle_jobs(
        &self,
        service: Arc<WebService>,
        state: Arc<Mutex<WorkerState>>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("handle_jobs")
            .and(WorkerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |jobs: Vec<Job>| {
                info!(
                    "#### Received {} handle_jobs request body {:?} ####",
                    &jobs.len(),
                    &jobs
                );
                let clone_service = service.clone();
                let clone_state = state.clone();
                async move { clone_service.handle_jobs(jobs, clone_state).await }
            })
    }
    fn create_route_cancel_jobs(
        &self,
        service: Arc<WebService>,
        state: Arc<Mutex<WorkerState>>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("cancel_jobs")
            .and(WorkerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |jobs: Vec<JobId>| {
                info!(
                    "#### Received {} cancel_jobs request body {:?} ####",
                    &jobs.len(),
                    &jobs
                );
                let clone_service = service.clone();
                let clone_state = state.clone();
                async move { clone_service.cancel_jobs(jobs, clone_state).await }
            })
    }

    fn create_route_cancel_plans(
        &self,
        service: Arc<WebService>,
        state: Arc<Mutex<WorkerState>>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("cancel_plans")
            .and(WorkerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |plans: Vec<PlanId>| {
                info!(
                    "#### Received {} plans_cancel request body {:?} ####",
                    &plans.len(),
                    &plans
                );
                let clone_service = service.clone();
                let clone_state = state.clone();
                async move { clone_service.cancel_plans(plans, clone_state).await }
            })
    }
    fn create_route_update_jobs(
        &self,
        service: Arc<WebService>,
        state: Arc<Mutex<WorkerState>>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("jobs_update")
            .and(WorkerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |jobs: Vec<Job>| {
                info!("#### Received update_jobs request body {:?} ####", &jobs);
                let clone_service = service.clone();
                let clone_state = state.clone();
                async move { clone_service.update_jobs(jobs, clone_state).await }
            })
    }
    fn create_route_get_state(
        &self,
        service: Arc<WebService>,
        state: Arc<Mutex<WorkerState>>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("get_state")
            .and(WorkerServer::log_headers())
            .and(warp::get())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |_param: WorkerStateParam| {
                info!("#### Received get_state request body ####");
                let clone_service = service.clone();
                let clone_state = state.clone();
                async move { clone_service.get_state(clone_state).await }
            })
    }

    fn log_headers() -> impl Filter<Extract = (), Error = Infallible> + Copy {
        warp::header::headers_cloned()
            .map(|headers: HeaderMap| {
                trace!("#### Received request header ####");
                for (k, v) in headers.iter() {
                    // Error from `to_str` should be handled properly
                    trace!(
                        "{}: {}",
                        k,
                        v.to_str().expect("Failed to print header value")
                    )
                }
            })
            .untuple_one()
    }
}
impl WebServerBuilder {
    pub fn with_entry_point(mut self, entry_point: &str) -> Self {
        self.entry_point = String::from(entry_point);
        self
    }
    pub fn with_access_control(mut self, access_control: AccessControl) -> Self {
        self.access_control = access_control;
        self
    }
    pub fn with_worker_state(mut self, worker_state: WorkerState) -> Self {
        self.worker_state = Arc::new(Mutex::new(worker_state));
        self
    }
    pub fn build(&self, service: WebService) -> WorkerServer {
        WorkerServer {
            entry_point: self.entry_point.clone(),
            access_control: self.access_control.clone(),
            web_service: Arc::new(service),
            worker_state: self.worker_state.clone(),
        }
    }
}

async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (StatusCode::BAD_REQUEST, "Payload too large".to_string())
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (
            StatusCode::BAD_REQUEST,
            format!("Authorization error, {:?}", err),
        )
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    Ok(warp::reply::with_status(message, code))
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Error;
    use common::logger::init_logger;
    use common::task_spawn;
    use reqwest::Client;

    use crate::models::job::JobBuffer;
    use crate::services::WebServiceBuilder;

    use common::jobs::JobResult;
    use serde_json::json;
    use std::time::Duration;
    use test_util::helper::load_env;
    use tokio::sync::mpsc::{channel, Receiver, Sender};

    use tokio::time::sleep;

    #[tokio::test]
    async fn test_api_ping_and_job_handler_fisherman() -> Result<(), Error> {
        load_env();
        // let _res = init_logger(&String::from("Testing-Fisherman"));
        let local_port: &str = "4042";
        let socket_addr = format!("0.0.0.0:{}", local_port);

        info!("Init http service ");

        let report_callback = "0.0.0.0:3031";
        let worker_id = "worker_id";
        info!(
            "Successfully register worker {}, report_callback: {}",
            &worker_id, report_callback
        );
        let job_buffer = Arc::new(Mutex::new(JobBuffer::new()));
        let service = WebServiceBuilder::new().build();
        let access_control = AccessControl::default();
        // Create job process thread
        let server = WebServerBuilder::default()
            .with_entry_point(socket_addr.as_str())
            .with_access_control(access_control)
            .with_worker_state(WorkerState::new(job_buffer.clone()))
            .build(service);
        info!("Start fisherman service ");

        task_spawn::spawn(async move {
            info!("Start service");
            server.serve().await;
        });

        // Test Ping
        sleep(Duration::from_secs(1)).await;
        let url = format!("http://localhost:{}/ping", local_port);
        let resp = reqwest::get(url).await?.text().await?;
        info!("{:#?}", resp);
        let resp: SimpleResponse = serde_json::from_str(&resp)?;

        assert_eq!(resp, SimpleResponse { success: true });

        // Test Job handler
        let url = format!("http://localhost:{}/handle_jobs", local_port);
        let body = r###"
        [
            {
                "job_id": "1",
                "job_name": "HttpRoundTripTime",
                "job_type": "HttpRequest",
                "plan_id": "plan_id",
                "component_type": "Node",
                "component_id": "63381276-b140-4422-8a1c-50e17838407a",
                "priority": 1,
                "expected_runtime":0,
                "parallelable": true,
                "timeout":3000,
                "component_url": "https://20.29.48.100/_node/63381276-b140-4422-8a1c-50e17838407a/ping",
                "repeat_number": 5,
                "interval": 1000,
                "header": {},
                "job_detail": {"Ping":{}},
                "phase": "Regular"
            }
        ]
                    "###;

        let client = Client::new();
        let resp = client.post(url).body(body).send().await?.text().await?;
        info!("res: {:#?}", resp);

        let resp: serde_json::value::Value = serde_json::from_str(&resp)?;
        let expect_resp = json!({
                "success": true,
                "result": [
                    "1"
                ]
        });

        assert_eq!(resp, expect_resp);

        Ok(())
    }
}
