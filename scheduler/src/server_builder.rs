use crate::server_config::AccessControl;
use log::{debug, info};
use serde::{Deserialize, Serialize};

use std::convert::Infallible;
use std::net::SocketAddr;

use serde_json::json;
use std::sync::Arc;

use warp::http::{HeaderMap, Method};

use crate::service::{ProcessorService, WebService};
use common::component::ComponentInfo;
use common::jobs::JobResult;

use warp::{Filter, Rejection};

use crate::handler::{handle_rejection, handle_route_reports, UnAuthorization};
use crate::state::{ProcessorState, SchedulerState};
use crate::{BUILD_VERSION, SCHEDULER_AUTHORIZATION};
use common::workers::WorkerInfo;

pub const MAX_JSON_BODY_SIZE: u64 = 1024 * 1024;

#[derive(Default)]
pub struct ServerBuilder {
    entry_point: String,
    access_control: AccessControl,
    scheduler_service: WebService,
    processor_service: ProcessorService,
    scheduler_state: Arc<SchedulerState>,
    processor_state: Arc<ProcessorState>,
}

pub struct SchedulerServer {
    entry_point: String,
    access_control: AccessControl,
    scheduler_service: Arc<WebService>,
    processor_service: Arc<ProcessorService>,
    scheduler_state: Arc<SchedulerState>,
    processor_state: Arc<ProcessorState>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeployParam {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SimpleResponse {
    pub success: bool,
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
        // let context_extractor = warp::any().and(
        //     warp::header::<String>("authorization")
        //         .map(|token: String| -> Result<(), Error> { Ok(()) })
        //         .or(warp::any().map(|| Err(anyhow!("No authorization"))))
        //         .unify(),
        // );

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
            .or(self.create_version().with(&cors))
            .or(self
                .create_route_worker_register(
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
            // .or(self
            //     .create_route_worker_pause(
            //         self.scheduler_service.clone(),
            //         self.scheduler_state.clone(),
            //     )
            //     .with(&cors))
            // .or(self
            //     .create_route_worker_resume(
            //         self.scheduler_service.clone(),
            //         self.scheduler_state.clone(),
            //     )
            //     .with(&cors))
            .recover(handle_rejection);

        let socket_addr: SocketAddr = self.entry_point.parse().unwrap();

        warp::serve(router).run(socket_addr).await;
    }
    /// Ping API
    fn create_ping(&self) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
        warp::path!("ping").and(warp::get()).map(move || {
            info!("Receive ping request");
            Ok(warp::reply::json(&SimpleResponse { success: true }))
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

    fn create_route_worker_register(
        &self,
        service: Arc<WebService>,
        state: Arc<SchedulerState>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("worker" / "register")
            .and(SchedulerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and(warp::header::<String>("authorization"))
            .and_then(move |worker_info: WorkerInfo, authorization: String| {
                let clone_service = service.clone();
                let clone_state = state.clone();
                async move {
                    if authorization == *SCHEDULER_AUTHORIZATION {
                        info!(
                            "#### Received register worker request body {:?} ####",
                            &worker_info
                        );

                        clone_service
                            .register_worker(worker_info, clone_state)
                            .await
                    } else {
                        Err(warp::reject::custom(UnAuthorization))
                    }
                }
            })
    }
    // fn create_route_worker_pause(
    //     &self,
    //     service: Arc<WebService>,
    //     state: Arc<SchedulerState>,
    // ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    //     warp::path!("worker" / "pause")
    //         .and(SchedulerServer::log_headers())
    //         .and(warp::post())
    //         .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
    //         .and_then(move |worker_info: WorkerInfo| {
    //             info!("#### Received request body {:?} ####", &worker_info);
    //             let clone_service = service.clone();
    //             let clone_state = state.clone();
    //             async move { clone_service.pause_worker(worker_info, clone_state).await }
    //         })
    // }
    // fn create_route_worker_resume(
    //     &self,
    //     service: Arc<WebService>,
    //     state: Arc<SchedulerState>,
    // ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    //     warp::path!("worker" / "resume")
    //         .and(SchedulerServer::log_headers())
    //         .and(warp::post())
    //         .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
    //         .and_then(move |worker_info: WorkerInfo| {
    //             info!("#### Received request body {:?} ####", &worker_info);
    //             let clone_service = service.clone();
    //             let clone_state = state.clone();
    //             async move { clone_service.resume_worker(worker_info, clone_state).await }
    //         })
    // }

    fn create_route_node_verify(
        &self,
        service: Arc<WebService>,
        state: Arc<SchedulerState>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("provider" / "verify")
            .and(SchedulerServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            // Temporary disable authorization for compatible with portal
            // .and(warp::header::<String>("authorization"))
            // .and_then(move |node_info: ComponentInfo, authorization: String| {
            .and_then(move |node_info: ComponentInfo| {
                let clone_service = service.clone();
                let clone_state = state.clone();
                async move {
                    // if authorization == *SCHEDULER_AUTHORIZATION {
                    if true {
                        info!("#### Received verify request body {:?} ####", &node_info);
                        Ok(clone_service.node_verify(node_info, clone_state).await)
                    } else {
                        Err(warp::reject::custom(UnAuthorization))
                    }
                }
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
            .and(warp::header::<String>("authorization"))
            .map(move |job_results: Vec<JobResult>, authorization: String| {
                (service.clone(), state.clone(), job_results, authorization)
            })
            .untuple_one()
            .and_then(handle_route_reports)
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
        self.scheduler_state = Arc::new(scheduler_state);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::providers::ProviderStorage;
    use crate::models::workers::WorkerInfoStorage;
    use crate::persistence::services::{JobResultService, JobService, PlanService, WorkerService};
    use crate::service::{ProcessorServiceBuilder, SchedulerServiceBuilder};

    use anyhow::Error;

    use common::task_spawn;
    use reqwest::Client;
    use sea_orm::{DatabaseBackend, MockDatabase};
    use std::env;

    use crate::models::job_result_cache::JobResultCache;
    use crate::service::delivery::CancelPlanBuffer;
    use crate::service::report_portal::ReportRecord;
    use chrono::FixedOffset;
    use common::logger::init_logger;
    use serde_json::json;
    use std::time::Duration;
    use test_util::helper::{load_env, mock_db_connection};
    use tokio::fs;
    use tokio::sync::Mutex;
    use tokio::time::{sleep, Instant};

    #[tokio::test]
    async fn test_api_ping_scheduler() -> Result<(), Error> {
        load_env();
        //let _res = init_logger(&String::from("Testing-Scheduler"));
        let local_port: &str = "3032";
        let socket_addr = format!("0.0.0.0:{}", local_port);
        let db_conn = MockDatabase::new(DatabaseBackend::Postgres);
        let _arc_conn = Arc::new(db_conn);

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
        load_env();
        let authorization = SCHEDULER_AUTHORIZATION.to_string();
        println!("authorization: {authorization}");
        //let _res = init_logger(&String::from("Testing-Scheduler"));
        let local_port: &str = "3033";
        let socket_addr = format!("0.0.0.0:{}", local_port);
        let callback_url = format!("http://127.0.0.1:{}/report", local_port);
        env::set_var("REPORT_CALLBACK", callback_url.clone());
        // Mock DB
        let db_conn = mock_db_connection();
        let arc_conn = Arc::new(db_conn);
        let plan_service = Arc::new(PlanService::new(arc_conn.clone()));
        let worker_service = Arc::new(WorkerService::new(arc_conn.clone()));

        let scheduler_service = SchedulerServiceBuilder::default().build();
        let processor_service = ProcessorServiceBuilder::default().build();
        let access_control = AccessControl::default();
        let worker_infos = Arc::new(WorkerInfoStorage::new(vec![]));
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
    "url":"http://192.168.1.30:3030/handle_jobs",
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

        // Test case: Lack of author header
        let resp = client
            .post(url.clone())
            .body(body)
            .send()
            .await?
            .text()
            .await?;
        info!("Lack of author header res: {:#?}", resp);
        assert_eq!(resp, "Internal Server Error");

        // Test case: had wrong author header
        let resp = client
            .post(url.clone())
            .body(body)
            .header("authorization", "wrong_authorization")
            .send()
            .await?
            .text()
            .await?;
        info!("had wrong author header res: {:#?}", resp);
        assert_eq!(resp, "Authorization error, Rejection(UnAuthorization)");

        // Test case: had correct author header
        let resp = client
            .post(url)
            .body(body)
            .header("authorization", &*SCHEDULER_AUTHORIZATION)
            .send()
            .await?
            .text()
            .await?;
        info!("had correct author header res: {:#?}", resp);

        let resp = serde_json::from_str::<serde_json::value::Value>(&resp)?;
        let expect_resp = json!({
            "worker_id": "worker_id",
            "report_callback": callback_url
        });
        println!("{} == {}", resp, expect_resp);
        assert_eq!(resp, expect_resp);

        Ok(())
    }

    //#[ignore]
    #[tokio::test]
    async fn test_api_report() -> Result<(), Error> {
        load_env();
        let authorization = SCHEDULER_AUTHORIZATION.to_string();
        //let _res = init_logger(&String::from("Testing-Scheduler"));
        //let authorization = "DEFAULT_SCHEDULER_AUTHORIZATION";
        println!("authorization: {authorization}");
        let local_port: &str = "3034";
        let socket_addr = format!("0.0.0.0:{}", local_port);
        // Mock DB
        let db_conn = mock_db_connection();
        let arc_conn = Arc::new(db_conn);
        let plan_service = Arc::new(PlanService::new(arc_conn.clone()));
        let worker_service = Arc::new(WorkerService::new(arc_conn.clone()));

        let scheduler_service = SchedulerServiceBuilder::default().build();
        let processor_service = ProcessorServiceBuilder::default().build();
        let access_control = AccessControl::default();
        let worker_infos = Arc::new(WorkerInfoStorage::new(vec![]));
        let provider_storage = Arc::new(ProviderStorage::default());
        let cancel_plans_buffer: Arc<Mutex<CancelPlanBuffer>> =
            Arc::new(Mutex::new(CancelPlanBuffer::default()));

        let scheduler_state = SchedulerState::new(
            arc_conn.clone(),
            plan_service.clone(),
            worker_service,
            worker_infos.clone(),
            provider_storage.clone(),
        );

        info!("Init http service ");
        let result_cache = Arc::new(JobResultCache::default());
        let job_service = Arc::new(JobService::new(arc_conn.clone()));
        let result_service = Arc::new(JobResultService::new(arc_conn.clone()));
        let processor_state = ProcessorState::new(
            arc_conn.clone(),
            result_cache.clone(),
            plan_service.clone(),
            job_service.clone(),
            result_service.clone(),
            worker_infos.clone(),
            cancel_plans_buffer.clone(),
        );
        let server = ServerBuilder::default()
            .with_entry_point(&socket_addr)
            .with_access_control(access_control)
            .with_scheduler_state(scheduler_state)
            .with_processor_state(processor_state)
            .build(scheduler_service, processor_service);
        task_spawn::spawn(async move {
            info!("Start service");
            server.serve().await;
        });

        // loop {
        //     sleep(Duration::from_secs(3)).await;
        // }
        sleep(Duration::from_secs(3)).await;

        // Test report
        let body = r###"
[
    {"plan_id":"regular-9cee993f-41bf-47c3-9e3c-c725976a33cd","job_id":"f11621b9-0f60-409a-81f5-77bdbacabcfc","job_name":"RoundTripTime","worker_id":"7c7da61c-aec7-45b1-9e32-7436d4721ce0","provider_id":"9cee993f-41bf-47c3-9e3c-c725976a33cd","provider_type":"Gateway","phase":"Regular","result_detail":{"HttpRequest":{"job":{"job_id":"f11621b9-0f60-409a-81f5-77bdbacabcfc","job_type":"HttpRequest","job_name":"RoundTripTime","plan_id":"regular-9cee993f-41bf-47c3-9e3c-c725976a33cd","component_id":"9cee993f-41bf-47c3-9e3c-c725976a33cd","component_type":"Gateway","priority":1,"expected_runtime":1657534340780,"parallelable":true,"timeout":3000,"component_url":"https://43.156.82.22/_rtt","repeat_number":999999999,"interval":5000,"header":{},"job_detail":{"HttpRequest":{"url":"https://43.156.82.22/_rtt","chain_info":{"chain":"eth","network":"mainnet"},"method":"get","headers":{},"body":"","response_type":"text","response_values":{}}},"phase":"Regular"},"response":{"request_timestamp":1657534340788,"response_duration":10000,"detail":{"Body":"29598000"},"http_code":200,"error_code":0,"message":"success"}}},"receive_timestamp":1657534340819,"chain_info":{"chain":"eth","network":"mainnet"}},
    {"plan_id":"regular-9cee993f-41bf-47c3-9e3c-c725976a33cd","job_id":"f11621b9-0f60-409a-81f5-77bdbacabcfc","job_name":"RoundTripTime","worker_id":"7c7da61c-aec7-45b1-9e32-7436d4721ce0","provider_id":"9cee993f-41bf-47c3-9e3c-c725976a33cd","provider_type":"Gateway","phase":"Regular","result_detail":{"HttpRequest":{"job":{"job_id":"f11621b9-0f60-409a-81f5-77bdbacabcfc","job_type":"HttpRequest","job_name":"RoundTripTime","plan_id":"regular-9cee993f-41bf-47c3-9e3c-c725976a33cd","component_id":"9cee993f-41bf-47c3-9e3c-c725976a33cd","component_type":"Gateway","priority":1,"expected_runtime":1657534340780,"parallelable":true,"timeout":3000,"component_url":"https://43.156.82.22/_rtt","repeat_number":999999999,"interval":5000,"header":{},"job_detail":{"HttpRequest":{"url":"https://43.156.82.22/_rtt","chain_info":{"chain":"eth","network":"mainnet"},"method":"get","headers":{},"body":"","response_type":"text","response_values":{}}},"phase":"Regular"},"response":{"request_timestamp":1657534340788,"response_duration":10000,"detail":{"Body":"29598000"},"http_code":200,"error_code":0,"message":"success"}}},"receive_timestamp":1657534340819,"chain_info":{"chain":"eth","network":"mainnet"}},
    {"plan_id":"regular-9cee993f-41bf-47c3-9e3c-c725976a33cd","job_id":"f11621b9-0f60-409a-81f5-77bdbacabcfc","job_name":"RoundTripTime","worker_id":"7c7da61c-aec7-45b1-9e32-7436d4721ce0","provider_id":"9cee993f-41bf-47c3-9e3c-c725976a33cd","provider_type":"Gateway","phase":"Regular","result_detail":{"HttpRequest":{"job":{"job_id":"f11621b9-0f60-409a-81f5-77bdbacabcfc","job_type":"HttpRequest","job_name":"RoundTripTime","plan_id":"regular-9cee993f-41bf-47c3-9e3c-c725976a33cd","component_id":"9cee993f-41bf-47c3-9e3c-c725976a33cd","component_type":"Gateway","priority":1,"expected_runtime":1657534340780,"parallelable":true,"timeout":3000,"component_url":"https://43.156.82.22/_rtt","repeat_number":999999999,"interval":5000,"header":{},"job_detail":{"HttpRequest":{"url":"https://43.156.82.22/_rtt","chain_info":{"chain":"eth","network":"mainnet"},"method":"get","headers":{},"body":"","response_type":"text","response_values":{}}},"phase":"Regular"},"response":{"request_timestamp":1657534340788,"response_duration":10000,"detail":{"Body":"29598000"},"http_code":200,"error_code":0,"message":"success"}}},"receive_timestamp":1657534340819,"chain_info":{"chain":"eth","network":"mainnet"}},
    {"plan_id":"regular-9cee993f-41bf-47c3-9e3c-c725976a33cd","job_id":"f11621b9-0f60-409a-81f5-77bdbacabcfc","job_name":"RoundTripTime","worker_id":"7c7da61c-aec7-45b1-9e32-7436d4721ce0","provider_id":"9cee993f-41bf-47c3-9e3c-c725976a33cd","provider_type":"Gateway","phase":"Regular","result_detail":{"HttpRequest":{"job":{"job_id":"f11621b9-0f60-409a-81f5-77bdbacabcfc","job_type":"HttpRequest","job_name":"RoundTripTime","plan_id":"regular-9cee993f-41bf-47c3-9e3c-c725976a33cd","component_id":"9cee993f-41bf-47c3-9e3c-c725976a33cd","component_type":"Gateway","priority":1,"expected_runtime":1657534340780,"parallelable":true,"timeout":3000,"component_url":"https://43.156.82.22/_rtt","repeat_number":999999999,"interval":5000,"header":{},"job_detail":{"HttpRequest":{"url":"https://43.156.82.22/_rtt","chain_info":{"chain":"eth","network":"mainnet"},"method":"get","headers":{},"body":"","response_type":"text","response_values":{}}},"phase":"Regular"},"response":{"request_timestamp":1657534340788,"response_duration":10000,"detail":{"Body":"29598000"},"http_code":200,"error_code":0,"message":"success"}}},"receive_timestamp":1657534340819,"chain_info":{"chain":"eth","network":"mainnet"}},
    {"plan_id":"regular-9cee993f-41bf-47c3-9e3c-c725976a33cd","job_id":"f11621b9-0f60-409a-81f5-77bdbacabcfc","job_name":"RoundTripTime","worker_id":"7c7da61c-aec7-45b1-9e32-7436d4721ce0","provider_id":"9cee993f-41bf-47c3-9e3c-c725976a33cd","provider_type":"Gateway","phase":"Regular","result_detail":{"HttpRequest":{"job":{"job_id":"f11621b9-0f60-409a-81f5-77bdbacabcfc","job_type":"HttpRequest","job_name":"RoundTripTime","plan_id":"regular-9cee993f-41bf-47c3-9e3c-c725976a33cd","component_id":"9cee993f-41bf-47c3-9e3c-c725976a33cd","component_type":"Gateway","priority":1,"expected_runtime":1657534340780,"parallelable":true,"timeout":3000,"component_url":"https://43.156.82.22/_rtt","repeat_number":999999999,"interval":5000,"header":{},"job_detail":{"HttpRequest":{"url":"https://43.156.82.22/_rtt","chain_info":{"chain":"eth","network":"mainnet"},"method":"get","headers":{},"body":"","response_type":"text","response_values":{}}},"phase":"Regular"},"response":{"request_timestamp":1657534340788,"response_duration":10000,"detail":{"Body":"29598000"},"http_code":200,"error_code":0,"message":"success"}}},"receive_timestamp":1657534340819,"chain_info":{"chain":"eth","network":"mainnet"}},
    {"plan_id":"regular-9cee993f-41bf-47c3-9e3c-c725976a33cd","job_id":"f11621b9-0f60-409a-81f5-77bdbacabcfc","job_name":"RoundTripTime","worker_id":"7c7da61c-aec7-45b1-9e32-7436d4721ce0","provider_id":"9cee993f-41bf-47c3-9e3c-c725976a33cd","provider_type":"Gateway","phase":"Regular","result_detail":{"HttpRequest":{"job":{"job_id":"f11621b9-0f60-409a-81f5-77bdbacabcfc","job_type":"HttpRequest","job_name":"RoundTripTime","plan_id":"regular-9cee993f-41bf-47c3-9e3c-c725976a33cd","component_id":"9cee993f-41bf-47c3-9e3c-c725976a33cd","component_type":"Gateway","priority":1,"expected_runtime":1657534340780,"parallelable":true,"timeout":3000,"component_url":"https://43.156.82.22/_rtt","repeat_number":999999999,"interval":5000,"header":{},"job_detail":{"HttpRequest":{"url":"https://43.156.82.22/_rtt","chain_info":{"chain":"eth","network":"mainnet"},"method":"get","headers":{},"body":"","response_type":"text","response_values":{}}},"phase":"Regular"},"response":{"request_timestamp":1657534340788,"response_duration":10000,"detail":{"Body":"29598000"},"http_code":200,"error_code":0,"message":"success"}}},"receive_timestamp":1657534340819,"chain_info":{"chain":"eth","network":"mainnet"}}
]
        "###;

        let client = Client::new();
        let url = format!("http://localhost:{}/report", local_port);
        let resp = client
            .post(url)
            .body(body)
            .header("authorization", &authorization)
            .send()
            .await;
        info!("send resp: {:?}", resp);
        let resp = resp?.text().await;
        info!("text resp: {:?}", resp);
        let resp = resp?;
        println!("res: {:#?}", resp);

        let resp: SimpleResponse = serde_json::from_str(&resp)?;

        assert_eq!(resp, SimpleResponse { success: true });
        let expect_output_line = r###"{"report_time":"","provider_id":"9cee993f-41bf-47c3-9e3c-c725976a33cd","plan_id":"","result":{"Failed":{"inner":[{"job_name":"HttpPing","failed_detail":"95% response duration : 29606 > 500"}]}}}"###;
        let expect_output: ReportRecord = serde_json::from_str(expect_output_line).unwrap();

        let now = Instant::now();
        loop {
            assert!(now.elapsed().as_secs() < 20);
            let data = fs::read_to_string("logs/report.txt")
                .await
                .expect("Unable to read file");

            let line = data.lines().rev().next().unwrap_or_default();
            let mut output: ReportRecord = serde_json::from_str(line).unwrap();
            output.report_time = "".to_string();
            if expect_output == output {
                info!("Reported the bad node!");
                assert!(true);
                break;
            }
            info!("report.txt: {}", data);
            sleep(Duration::from_secs(1)).await;
        }
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_verification() -> Result<(), Error> {
        load_env();
        let authorization = SCHEDULER_AUTHORIZATION.to_string();
        //let _res = init_logger(&String::from("Testing-Scheduler"));
        //let authorization = "DEFAULT_SCHEDULER_AUTHORIZATION";
        println!("authorization: {authorization}");
        let local_port: &str = "3034";
        let socket_addr = format!("0.0.0.0:{}", local_port);
        // Mock DB
        let db_conn = mock_db_connection();
        let arc_conn = Arc::new(db_conn);
        let plan_service = Arc::new(PlanService::new(arc_conn.clone()));
        let worker_service = Arc::new(WorkerService::new(arc_conn.clone()));

        let scheduler_service = SchedulerServiceBuilder::default().build();
        let processor_service = ProcessorServiceBuilder::default().build();
        let access_control = AccessControl::default();
        let worker_infos = Arc::new(WorkerInfoStorage::new(vec![]));
        let provider_storage = Arc::new(ProviderStorage::default());
        let cancel_plans_buffer: Arc<Mutex<CancelPlanBuffer>> =
            Arc::new(Mutex::new(CancelPlanBuffer::default()));
        let scheduler_state = SchedulerState::new(
            arc_conn.clone(),
            plan_service.clone(),
            worker_service,
            worker_infos.clone(),
            provider_storage.clone(),
        );

        info!("Init http service ");
        let result_cache = Arc::new(JobResultCache::default());
        let job_service = Arc::new(JobService::new(arc_conn.clone()));
        let result_service = Arc::new(JobResultService::new(arc_conn.clone()));
        let processor_state = ProcessorState::new(
            arc_conn.clone(),
            result_cache.clone(),
            plan_service.clone(),
            job_service.clone(),
            result_service.clone(),
            worker_infos.clone(),
            cancel_plans_buffer.clone(),
        );
        let server = ServerBuilder::default()
            .with_entry_point(&socket_addr)
            .with_access_control(access_control)
            .with_scheduler_state(scheduler_state)
            .with_processor_state(processor_state)
            .build(scheduler_service, processor_service);
        task_spawn::spawn(async move {
            info!("Start service");
            server.serve().await;
        });

        // loop {
        //     sleep(Duration::from_secs(3)).await;
        // }
        sleep(Duration::from_secs(3)).await;

        // Test verification
        let body = r###"
{
    "appKey":"lSP1lFN9I_izEzRi_jBapA",
    "blockchain":"eth",
    "componentType":"Node",
    "countryCode":"US",
    "id":"058a6e94-8b65-46ad-ab52-240a7cb2c36a",
    "ip":"34.101.146.31",
    "name":"node-net-highcpu2-asia-southeast2-a-04",
    "network":"mainnet",
    "userId":"b363ddf4-42cf-4ccf-89c2-8c42c531ac99",
    "zone":"AS"
}
        "###;

        let client = Client::new();
        let url = format!("http://localhost:{}/provider/verify", local_port);
        let resp = client
            .post(url)
            .body(body)
            .header("authorization", &authorization)
            .send()
            .await?
            .text()
            .await?;
        info!("res: {:#?}", resp);

        let resp: SimpleResponse = serde_json::from_str(&resp)?;

        assert_eq!(resp, SimpleResponse { success: true });

        Ok(())
    }

    #[tokio::test]
    async fn test_version() {
        let _res = init_logger(&String::from("Testing-Scheduler"), None);
        info!("***time");
        let time = chrono::offset::Local::now()
            .with_timezone(&FixedOffset::east(7 * 60 * 60))
            .to_string();
        println!("time: {}", time);
    }
}
