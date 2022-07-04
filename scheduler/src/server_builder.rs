use crate::server_config::AccessControl;
use common::job_manage::JobResultDetail;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::borrow::BorrowMut;
use std::collections::VecDeque;

use serde_json::{json, Value};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc::Sender, Mutex};
use warp::http::{HeaderMap, Method};

use crate::service::{ProcessorService, WebService};
use common::component::ComponentInfo;
use common::jobs::JobResult;
use common::task_spawn::{spawn, spawn_thread};
use warp::reply::Json;
use warp::{http::StatusCode, Filter, Rejection, Reply};

use crate::state::{ProcessorState, SchedulerState};
use common::workers::WorkerInfo;

static PROCESS_THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);
pub const MAX_JSON_BODY_SIZE: u64 = 1024 * 1024;

#[derive(Default)]
pub struct ServerBuilder {
    entry_point: String,
    access_control: AccessControl,
    scheduler_service: WebService,
    processor_service: ProcessorService,
    scheduler_state: Arc<Mutex<SchedulerState>>,
    processor_state: Arc<ProcessorState>,
}

pub struct SchedulerServer {
    entry_point: String,
    access_control: AccessControl,
    scheduler_service: Arc<WebService>,
    processor_service: Arc<ProcessorService>,
    scheduler_state: Arc<Mutex<SchedulerState>>,
    processor_state: Arc<ProcessorState>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeployParam {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SimpleResponse {
    success: bool,
}

impl SchedulerServer {
    pub fn builder() -> ServerBuilder {
        ServerBuilder {
            entry_point: "".to_string(),
            access_control: Default::default(),
            scheduler_service: Default::default(),
            processor_service: Default::default(),
            scheduler_state: Arc::new(Default::default()),
            processor_state: Arc::new(Default::default()),
        }
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
            .or(self
                .create_route_worker_register(
                    self.scheduler_service.clone(),
                    self.scheduler_state.clone(),
                )
                .with(&cors))
            .or(self
                .create_route_worker_pause(
                    self.scheduler_service.clone(),
                    self.scheduler_state.clone(),
                )
                .with(&cors))
            .or(self
                .create_route_worker_resume(
                    self.scheduler_service.clone(),
                    self.scheduler_state.clone(),
                )
                .with(&cors))
            .or(self
                .create_route_node_verify(
                    self.scheduler_service.clone(),
                    self.scheduler_state.clone(),
                )
                .with(&cors))
            //For report processor
            .or(self
                .create_route_reports(self.processor_service.clone(), self.processor_state.clone())
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
    fn create_route_worker_register(
        &self,
        service: Arc<WebService>,
        state: Arc<Mutex<SchedulerState>>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("worker" / "register")
            .and(SchedulerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |worker_info: WorkerInfo| {
                info!("#### Received request body {:?} ####", &worker_info);
                let clone_service = service.clone();
                let clone_state = state.clone();
                async move {
                    clone_service
                        .register_worker(worker_info, clone_state)
                        .await
                }
            })
    }
    fn create_route_worker_pause(
        &self,
        service: Arc<WebService>,
        state: Arc<Mutex<SchedulerState>>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("worker" / "pause")
            .and(SchedulerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |worker_info: WorkerInfo| {
                info!("#### Received request body {:?} ####", &worker_info);
                let clone_service = service.clone();
                let clone_state = state.clone();
                async move { clone_service.pause_worker(worker_info, clone_state).await }
            })
    }
    fn create_route_worker_resume(
        &self,
        service: Arc<WebService>,
        state: Arc<Mutex<SchedulerState>>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("worker" / "resume")
            .and(SchedulerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |worker_info: WorkerInfo| {
                info!("#### Received request body {:?} ####", &worker_info);
                let clone_service = service.clone();
                let clone_state = state.clone();
                async move { clone_service.resume_worker(worker_info, clone_state).await }
            })
    }
    fn create_route_worker_stop(
        &self,
        service: Arc<WebService>,
        state: Arc<Mutex<SchedulerState>>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("worker" / "stop")
            .and(SchedulerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |worker_info: WorkerInfo| {
                info!("#### Received request body {:?} ####", &worker_info);
                let clone_service = service.clone();
                let clone_state = state.clone();
                async move { clone_service.stop_worker(worker_info, clone_state).await }
            })
    }
    fn create_route_node_verify(
        &self,
        service: Arc<WebService>,
        state: Arc<Mutex<SchedulerState>>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("node" / "verify")
            .and(SchedulerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |node_info: ComponentInfo| {
                info!("#### Received request body {:?} ####", &node_info);
                let clone_service = service.clone();
                let clone_state = state.clone();
                async move { clone_service.node_verify(node_info, clone_state).await }
            })
    }
    fn create_route_reports(
        &self,
        service: Arc<ProcessorService>,
        state: Arc<ProcessorState>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("report")
            .and(SchedulerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |job_results: Vec<JobResult>| {
                info!(
                    "#### Received {:?} reports request body  ####",
                    &job_results.len()
                );
                let clone_service = service.clone();
                let clone_state = state.clone();
                spawn(async move {
                    PROCESS_THREAD_COUNT.fetch_add(1, Ordering::Relaxed);
                    let job_results_len = job_results.len();
                    let now = Instant::now();
                    info!(
                        "** Start {}th process {} job results **",
                        PROCESS_THREAD_COUNT.load(Ordering::Relaxed),
                        job_results_len
                    );
                    clone_service.process_report(job_results, clone_state).await;
                    info!(
                        "** Finished process {} job results in {:.2?} **",
                        job_results_len,
                        now.elapsed()
                    );
                    PROCESS_THREAD_COUNT.fetch_sub(1, Ordering::Relaxed);
                });

                async move { Self::simple_response(true).await }
            })
    }

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
impl ServerBuilder {
    pub fn with_entry_point(mut self, entry_point: &str) -> Self {
        self.entry_point = String::from(entry_point);
        self
    }
    pub fn with_access_control(mut self, access_control: AccessControl) -> Self {
        self.access_control = access_control;
        self
    }
    pub fn with_scheduler_service(mut self, scheduler_service: WebService) -> Self {
        self.scheduler_service = scheduler_service;
        self
    }
    pub fn with_processor_service(mut self, processor_service: ProcessorService) -> Self {
        self.processor_service = processor_service;
        self
    }
    pub fn with_scheduler_state(mut self, scheduler_state: SchedulerState) -> Self {
        self.scheduler_state = Arc::new(Mutex::new(scheduler_state));
        self
    }
    pub fn with_processor_state(mut self, processor_state: ProcessorState) -> Self {
        self.processor_state = Arc::new(processor_state);
        self
    }
    pub fn build(&self, scheduler: WebService, processor: ProcessorService) -> SchedulerServer {
        SchedulerServer {
            entry_point: self.entry_point.clone(),
            access_control: self.access_control.clone(),
            scheduler_service: Arc::new(scheduler),
            processor_service: Arc::new(processor),
            scheduler_state: self.scheduler_state.clone(),
            processor_state: self.processor_state.clone(),
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
