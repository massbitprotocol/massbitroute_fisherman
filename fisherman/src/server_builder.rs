use crate::server_config::AccessControl;
use common::jobs::Job;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use serde_json::Value;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use warp::http::{HeaderMap, Method};

use crate::services::WebService;
use crate::state::WorkerState;
use common::workers::WorkerInfo;
use common::workers::WorkerStateParam;
use std::default::Default;
use tokio::sync::Mutex;
use warp::reply::Json;
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

#[derive(Debug, Deserialize, Serialize, Clone)]
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
    pub(crate) async fn simple_response(success: bool) -> Result<impl Reply, Rejection> {
        let res = SimpleResponse { success };
        Ok(warp::reply::json(&res))
    }
    fn create_route_handle_jobs(
        &self,
        service: Arc<WebService>,
        state: Arc<Mutex<WorkerState>>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("jobs_handle")
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
            .and_then(move |param: WorkerStateParam| {
                info!("#### Received request body ####");
                let clone_service = service.clone();
                let clone_state = state.clone();
                async move { clone_service.get_state(clone_state).await }
            })
    }

    /// Get status of component
    /*
    async fn create_get_status(
        &self,
        service: Arc<Scheduler>,
        sender: Sender<ComponentInfo>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let sender_clone = sender.clone();
        warp::path!("get_status")
            .and(SchedulerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |body: Value| {
                info!("#### Received request body ####");
                info!("{}", body);
                let component_info: ComponentInfo = serde_json::from_value(body).unwrap();
                let sender_another_clone = sender_clone.clone();
                async move {
                    let res = sender_another_clone.send(component_info).await;
                    Self::simple_response(true).await
                }
            })
    }
    */

    fn log_headers() -> impl Filter<Extract = (), Error = Infallible> + Copy {
        warp::header::headers_cloned()
            .map(|headers: HeaderMap| {
                debug!("#### Received request header ####");
                for (k, v) in headers.iter() {
                    // Error from `to_str` should be handled properly
                    debug!(
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
