use crate::server_config::AccessControl;
use common::jobs::JobResult;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::borrow::BorrowMut;
use std::collections::VecDeque;

use serde_json::Value;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;
use warp::http::{HeaderMap, Method};

use crate::service::{ProcessorService, SchedulerService};
use warp::reply::Json;
use warp::{http::StatusCode, Filter, Rejection, Reply};

use crate::state::{ProcessorState, SchedulerState};
use common::models::WorkerInfo;

pub const MAX_JSON_BODY_SIZE: u64 = 1024 * 1024;

#[derive(Default)]
pub struct ServerBuilder {
    entry_point: String,
    access_control: AccessControl,
}

pub struct SchedulerServer {
    entry_point: String,
    access_control: AccessControl,
    scheduler_service: Arc<SchedulerService>,
    processor_service: Arc<ProcessorService>,
    scheduler_state: Arc<Mutex<SchedulerState>>,
    processor_state: Arc<Mutex<ProcessorState>>,
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
        ServerBuilder::default()
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
                .create_route_register_worker(self.scheduler_service.clone())
                .with(&cors))
            .or(self
                .create_route_pause_worker(self.scheduler_service.clone())
                .with(&cors))
            .or(self
                .create_route_resume_worker(self.scheduler_service.clone())
                .with(&cors))
            //For report processor
            .or(self
                .create_route_reports(self.processor_service.clone())
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
    fn create_route_register_worker(
        &self,
        service: Arc<SchedulerService>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("worker" / "register")
            .and(SchedulerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |worker_info: WorkerInfo| {
                info!("#### Received request body {:?} ####", &worker_info);
                let clone_service = service.clone();
                // let component_info: ComponentInfo = serde_json::from_value(body).unwrap();
                // let sender_another_clone = sender_clone.clone();
                async move { clone_service.register_worker(worker_info).await }
            })
    }
    fn create_route_pause_worker(
        &self,
        service: Arc<SchedulerService>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("worker" / "pause")
            .and(SchedulerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |worker_info: WorkerInfo| {
                info!("#### Received request body {:?} ####", &worker_info);
                let clone_service = service.clone();
                // let component_info: ComponentInfo = serde_json::from_value(body).unwrap();
                // let sender_another_clone = sender_clone.clone();
                async move { clone_service.pause_worker(worker_info).await }
            })
    }
    fn create_route_resume_worker(
        &self,
        service: Arc<SchedulerService>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("worker" / "resume")
            .and(SchedulerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |worker_info: WorkerInfo| {
                info!("#### Received request body {:?} ####", &worker_info);
                let clone_service = service.clone();
                // let component_info: ComponentInfo = serde_json::from_value(body).unwrap();
                // let sender_another_clone = sender_clone.clone();
                async move { clone_service.resume_worker(worker_info).await }
            })
    }
    fn create_route_stop_worker(
        &self,
        service: Arc<SchedulerService>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("worker" / "stop")
            .and(SchedulerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |worker_info: WorkerInfo| {
                info!("#### Received request body {:?} ####", &worker_info);
                let clone_service = service.clone();
                // let component_info: ComponentInfo = serde_json::from_value(body).unwrap();
                // let sender_another_clone = sender_clone.clone();
                async move { clone_service.resume_worker(worker_info).await }
            })
    }

    fn create_route_reports(
        &self,
        service: Arc<ProcessorService>,
        state: Arc<Mutex<ProcessorState>>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("report")
            .and(SchedulerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |worker_info: WorkerInfo| {
                info!("#### Received request body {:?} ####", &worker_info);
                let clone_service = service.clone();
                let clone_state = state.clone();
                async move { clone_service.process_report(worker_info, clone_state).await }
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
    pub fn build(
        &self,
        scheduler: SchedulerService,
        processor: ProcessorService,
    ) -> SchedulerServer {
        SchedulerServer {
            entry_point: self.entry_point.clone(),
            access_control: self.access_control.clone(),
            scheduler_service: Arc::new(scheduler),
            processor_service: Arc::new(processor),
            scheduler_state: Arc::new(Mutex::new(SchedulerState {})),
            processor_state: Arc::new(Mutex::new(ProcessorState {})),
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
