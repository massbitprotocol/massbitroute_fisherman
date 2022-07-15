use crate::server_config::AccessControl;
use log::{debug, info};
use serde::{Deserialize, Serialize};

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use warp::http::{HeaderMap, Method};

use crate::service::{ProcessorService, WebService};
use common::component::ComponentInfo;
use common::jobs::JobResult;
use common::task_spawn::spawn;
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
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
        warp::path!("provider" / "verify")
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
                    let res = clone_service.process_report(job_results, clone_state).await;
                    info!(
                        "** Finished process {} job results in {:.2?} with res: {:?} **",
                        job_results_len,
                        now.elapsed(),
                        res
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::providers::ProviderStorage;
    use crate::models::workers::WorkerInfoStorage;
    use crate::persistence::services::{get_sea_db_connection, PlanService, WorkerService};
    use crate::service::{ProcessorServiceBuilder, SchedulerServiceBuilder};
    use crate::{DATABASE_URL, SCHEDULER_ENDPOINT};
    use anyhow::Error;
    use common::logger::init_logger;
    use common::task_spawn;
    use reqwest::Client;
    use sea_orm::{
        entity::prelude::*, entity::*, tests_cfg::*, DatabaseBackend, MockDatabase, Transaction,
    };
    use std::env;

    use serde_json::json;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_api_ping() -> Result<(), Error> {
        let _res = init_logger(&String::from("Testing-Scheduler"));
        let local_port: &str = "3032";
        let socket_addr = format!("0.0.0.0:{}", local_port);
        let db_conn = MockDatabase::new(DatabaseBackend::Postgres);
        let arc_conn = Arc::new(db_conn);

        let scheduler_service = SchedulerServiceBuilder::default().build();
        let processor_service = ProcessorServiceBuilder::default().build();
        let access_control = AccessControl::default();

        info!("Init http service ");

        let server = ServerBuilder::default()
            .with_entry_point(&socket_addr)
            .with_access_control(access_control)
            .build(scheduler_service, processor_service);
        task_spawn::spawn(async move {
            info!("Start service");
            server.serve().await;
        });

        sleep(Duration::from_secs(1)).await;
        let url = format!("http://localhost:{}/ping", local_port);
        let resp = reqwest::get(url).await?.text().await?;
        info!("{:#?}", resp);
        let resp: SimpleResponse = serde_json::from_str(&resp)?;

        assert_eq!(resp, SimpleResponse { success: true });

        Ok(())
    }

    #[tokio::test]
    async fn test_api_worker_register() -> Result<(), Error> {
        dotenv::from_filename(".env_test").ok();
        //let _res = init_logger(&String::from("Testing-Scheduler"));
        let local_port: &str = "3033";
        let socket_addr = format!("0.0.0.0:{}", local_port);
        let callback_url = format!("http://127.0.0.1:{}/report", local_port);
        env::set_var("REPORT_CALLBACK", callback_url.clone());
        // Mock DB
        let db_conn = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
        let arc_conn = Arc::new(db_conn);
        let plan_service = Arc::new(PlanService::new(arc_conn.clone()));
        let worker_service = Arc::new(WorkerService::new(arc_conn.clone()));

        let scheduler_service = SchedulerServiceBuilder::default().build();
        let processor_service = ProcessorServiceBuilder::default().build();
        let access_control = AccessControl::default();
        let worker_infos = Arc::new(Mutex::new(WorkerInfoStorage::new(vec![])));
        let provider_storage = Arc::new(ProviderStorage::default());

        let scheduler_state = SchedulerState::new(
            arc_conn.clone(),
            plan_service.clone(),
            worker_service,
            worker_infos.clone(),
            provider_storage.clone(),
        );

        info!("Init http service ");

        let server = ServerBuilder::default()
            .with_entry_point(&socket_addr)
            .with_access_control(access_control)
            .with_scheduler_state(scheduler_state)
            .build(scheduler_service, processor_service);
        task_spawn::spawn(async move {
            info!("Start service");
            server.serve().await;
        });

        sleep(Duration::from_secs(1)).await;
        let body = r###"
{
    "worker_id":"worker_id",
    "app_key":"lSP1lFN9I_izEzRi_jBapA",
    "worker_ip":"192.168.1.30",
    "url":"http://192.168.1.30:3030/jobs_handle",
    "zone":"AS",
    "worker_spec": {
        "cpus": 4,
        "ram": 32,
        "bandwidth": 128
    }
}
        "###;

        let client = Client::new();
        let url = format!("http://localhost:{}/worker/register", local_port);
        let resp = client.post(url).body(body).send().await?.text().await?;
        info!("res: {:#?}", resp);

        let resp: serde_json::value::Value = serde_json::from_str(&resp)?;
        let expect_resp = json!({
            "worker_id": "worker_id",
            "report_callback": callback_url
        });

        assert_eq!(resp, expect_resp);

        Ok(())
    }

    #[tokio::test]
    async fn test_api_report() -> Result<(), Error> {
        dotenv::from_filename(".env_test").ok();
        //let _res = init_logger(&String::from("Testing-Scheduler"));
        let local_port: &str = "3034";
        let socket_addr = format!("0.0.0.0:{}", local_port);
        // Mock DB
        let db_conn = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
        let arc_conn = Arc::new(db_conn);
        let plan_service = Arc::new(PlanService::new(arc_conn.clone()));
        let worker_service = Arc::new(WorkerService::new(arc_conn.clone()));

        let scheduler_service = SchedulerServiceBuilder::default().build();
        let processor_service = ProcessorServiceBuilder::default().build();
        let access_control = AccessControl::default();
        let worker_infos = Arc::new(Mutex::new(WorkerInfoStorage::new(vec![])));
        let provider_storage = Arc::new(ProviderStorage::default());

        let scheduler_state = SchedulerState::new(
            arc_conn.clone(),
            plan_service.clone(),
            worker_service,
            worker_infos.clone(),
            provider_storage.clone(),
        );

        info!("Init http service ");

        let server = ServerBuilder::default()
            .with_entry_point(&socket_addr)
            .with_access_control(access_control)
            .with_scheduler_state(scheduler_state)
            .build(scheduler_service, processor_service);
        task_spawn::spawn(async move {
            info!("Start service");
            server.serve().await;
        });

        sleep(Duration::from_secs(1)).await;
        let body = r###"
[
    {"plan_id":"regular-9dafe6e7-460f-4344-b5a7-eeb1d9a0574d","job_id":"b0b7cf69-2ecc-4282-8a95-488d7e3b04a9","job_name":"RoundTripTime","worker_id":"7c7da61c-aec7-45b1-9e32-7436d4721ce0","provider_id":"9dafe6e7-460f-4344-b5a7-eeb1d9a0574d","provider_type":"Gateway","phase":"Regular","result_detail":{"HttpRequest":{"job":{"job_id":"b0b7cf69-2ecc-4282-8a95-488d7e3b04a9","job_type":"HttpRequest","job_name":"RoundTripTime","plan_id":"regular-9dafe6e7-460f-4344-b5a7-eeb1d9a0574d","component_id":"9dafe6e7-460f-4344-b5a7-eeb1d9a0574d","component_type":"Gateway","priority":1,"expected_runtime":1657534340780,"parallelable":true,"timeout":3000,"component_url":"https://43.156.248.214/_rtt","repeat_number":999999999,"interval":5000,"header":{},"job_detail":{"HttpRequest":{"url":"https://43.156.248.214/_rtt","chain_info":{"chain":"eth","network":"mainnet"},"method":"get","headers":{},"body":"","response_type":"text","response_values":{}}},"phase":"Regular"},"response":{"request_timestamp":1657534340788,"response_duration":26,"detail":{"Body":"25085"},"http_code":200,"error_code":0,"message":"success"}}},"receive_timestamp":1657534340814,"chain_info":{"chain":"eth","network":"mainnet"}},
    {"plan_id":"regular-9cee993f-41bf-47c3-9e3c-c725976a33cd","job_id":"f11621b9-0f60-409a-81f5-77bdbacabcfc","job_name":"RoundTripTime","worker_id":"7c7da61c-aec7-45b1-9e32-7436d4721ce0","provider_id":"9cee993f-41bf-47c3-9e3c-c725976a33cd","provider_type":"Gateway","phase":"Regular","result_detail":{"HttpRequest":{"job":{"job_id":"f11621b9-0f60-409a-81f5-77bdbacabcfc","job_type":"HttpRequest","job_name":"RoundTripTime","plan_id":"regular-9cee993f-41bf-47c3-9e3c-c725976a33cd","component_id":"9cee993f-41bf-47c3-9e3c-c725976a33cd","component_type":"Gateway","priority":1,"expected_runtime":1657534340780,"parallelable":true,"timeout":3000,"component_url":"https://43.156.82.22/_rtt","repeat_number":999999999,"interval":5000,"header":{},"job_detail":{"HttpRequest":{"url":"https://43.156.82.22/_rtt","chain_info":{"chain":"eth","network":"mainnet"},"method":"get","headers":{},"body":"","response_type":"text","response_values":{}}},"phase":"Regular"},"response":{"request_timestamp":1657534340788,"response_duration":31,"detail":{"Body":"29598"},"http_code":200,"error_code":0,"message":"success"}}},"receive_timestamp":1657534340819,"chain_info":{"chain":"eth","network":"mainnet"}}
]
        "###;

        let client = Client::new();
        let url = format!("http://localhost:{}/report", local_port);
        let resp = client.post(url).body(body).send().await?.text().await?;
        info!("res: {:#?}", resp);

        let resp: SimpleResponse = serde_json::from_str(&resp)?;

        assert_eq!(resp, SimpleResponse { success: true });

        Ok(())
    }
}
