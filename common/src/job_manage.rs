use anyhow::Error;
use log::info;
use std::collections::HashMap;
use std::hash::Hash;
use std::str::FromStr;

use crate::component::ComponentType;
use crate::jobs::Job;
use crate::tasks::command::{JobCommand, JobCommandResult};
use crate::tasks::compound::JobCompound;
use crate::tasks::eth::{CallBenchmarkError, JobLatestBlock, JobLatestBlockResult};
use crate::tasks::http_request::{JobHttpRequest, JobHttpResult};
use crate::tasks::ping::JobPingResult;
use crate::tasks::rpc_request::{JobRpcRequest, JobRpcResult};
use crate::tasks::websocket_request::{JobWebsocket, JobWebsocketResult};
use crate::{BlockChainType, JobId, Timestamp, WorkerId};
use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, Hash, Eq)]
pub enum JobType {
    // perform ping check
    PING,
    // Perform some request to node/gateway
    REQUEST,
    // perform benchmark checking
    BENCHMARK,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobCancel {
    /*
     * Time to perform ping request (if duration =-1 then perform ping without finish)
     */
    job_id: JobId,
    reason: String, //Using for log
}
#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct JobPing {}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct JobBenchmark {
    pub component_type: ComponentType,
    pub chain_type: BlockChainType,
    pub connection: u32,
    pub thread: u32,
    pub rate: u32,            // Requests/sec
    pub duration: Timestamp,  // Time to perform benchmark in ms
    pub script: String,       // Name of .lua script
    pub histograms: Vec<u32>, // List of expected percentile,
    pub url_path: String,     // URL path for benchmark
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobCancelResult {
    /*
     * Time to perform ping request (if duration =-1 then perform ping without finish)
     */
    job_id: JobId,
    status: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobCompoundResult {
    job: Job,
    response_timestamp: Timestamp, //Time to get response
    response_status: String,       //http status
    values: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobBenchmarkResult {
    pub job: Job,
    pub worker_id: WorkerId,
    pub response_timestamp: Timestamp, //Time to get response
    pub response: BenchmarkResponse,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct BenchmarkResponse {
    pub request_rate: f32,
    pub transfer_rate: f32,   //KB
    pub average_latency: f32, //In ms
    pub histograms: HashMap<u32, f32>,
    pub error_code: u32,
    pub message: String,
}

impl From<CallBenchmarkError> for BenchmarkResponse {
    fn from(error: CallBenchmarkError) -> Self {
        BenchmarkResponse::new_error(error.get_code(), error.get_message().as_str())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum JobDetail {
    HttpRequest(JobHttpRequest),
    RpcRequest(JobRpcRequest),
    Command(JobCommand),
    Websocket(JobWebsocket),
    // Perform some request to node/gaJobteway
    Compound(JobCompound),
    // perform ping check
    Ping(JobPing),
    LatestBlock(JobLatestBlock),
    // perform benchmark checking
    Benchmark(JobBenchmark),
}

impl JobDetail {
    pub fn get_job_name(&self) -> String {
        match self {
            JobDetail::HttpRequest(_) => "HttpRequest".to_string(),
            JobDetail::RpcRequest(_) => "RpcRequest".to_string(),
            JobDetail::Command(_) => "Command".to_string(),
            JobDetail::Websocket(_) => "Websocket".to_string(),
            JobDetail::Compound(_) => "Compound".to_string(),
            JobDetail::Ping(_) => "Ping".to_string(),
            JobDetail::LatestBlock(_) => "LatestBlock".to_string(),
            JobDetail::Benchmark(_) => "Benchmark".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum JobResultDetail {
    HttpRequest(JobHttpResult),
    Websocket(JobWebsocketResult),
    RpcRequest(JobRpcResult),
    Command(JobCommandResult),
    // perform ping check
    Ping(JobPingResult),
    LatestBlock(JobLatestBlockResult),
    // perform benchmark checking
    Benchmark(JobBenchmarkResult),
    // Perform some request to node/gateway
    Compound(JobCompoundResult),
}

impl JobResultDetail {
    /*
    pub fn new(job: &Job) -> Self {
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        match job.job_detail.as_ref().unwrap() {
            JobDetail::HttpRequest(_rpc) => JobResultDetail::HttpRequest(JobHttpResult::new(
                job.clone(),
                JobHttpResponse::default(),
            )),
            JobDetail::RpcRequest(_rpc) => JobResultDetail::RpcRequest(JobRpcResult::new(
                job.clone(),
                JobRpcResponse::default(),
            )),
            JobDetail::Command(_rpc) => JobResultDetail::Command(JobCommandResult::new(
                job.clone(),
                JobCommandResponse::default(),
            )),
            JobDetail::Websocket(_rpc) => JobResultDetail::Websocket(JobWebsocketResult::new(
                job.clone(),
                JobWebsocketResponse::default(),
            )),
            JobDetail::Ping(_) => JobResultDetail::Ping(JobPingResult {
                job: job.clone(),
                //response_timestamp: current_timestamp,
                ..Default::default()
            }),
            JobDetail::LatestBlock(_) => JobResultDetail::LatestBlock(JobLatestBlockResult {
                job: job.clone(),
                //response_timestamp: current_timestamp,
                ..Default::default()
            }),
            JobDetail::Compound(_) => JobResultDetail::Compound(JobCompoundResult {
                job: job.clone(),
                response_timestamp: current_timestamp as i64,
                ..Default::default()
            }),
            JobDetail::Benchmark(_) => JobResultDetail::Benchmark(JobBenchmarkResult {
                job: job.clone(),
                response_timestamp: current_timestamp as i64,
                ..Default::default()
            }),
        }
    }
     */
    pub async fn send(&self) -> Result<String, Error> {
        //http://192.168.1.30:3031/report
        let url = "http://192.168.1.30:3031/report";

        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        // Replace body for transport result of previous step
        let body = serde_json::to_string(self)?;

        info!("body: {:?}", body);
        let request_builder = client
            .post(url)
            .header("content-type", "application/json")
            .body(body);
        info!("request_builder: {:?}", request_builder);

        let sender = request_builder.send().await?.text().await?;

        Ok(sender)
    }
    /*
    pub fn get_job(&self) -> &Job {
        match self {
            JobResultDetail::HttpRequest(job_result) => &job_result.job,
            JobResultDetail::RpcRequest(job_result) => &job_result.job,
            JobResultDetail::Websocket(job_result) => &job_result.job,
            JobResultDetail::Command(job_result) => &job_result.job,
            JobResultDetail::Ping(job_result) => &job_result.job,
            JobResultDetail::LatestBlock(job_result) => &job_result.job,
            JobResultDetail::Benchmark(job_result) => &job_result.job,
            JobResultDetail::Compound(job_result) => &job_result.job,
        }
    }
     */
}

impl JobResultDetail {
    pub fn get_name(&self) -> String {
        match self {
            JobResultDetail::HttpRequest(_) => "HttpRequest".to_string(),
            JobResultDetail::RpcRequest(_) => "RpcRequest".to_string(),
            JobResultDetail::Command(_) => "Command".to_string(),
            JobResultDetail::Ping(_) => "Ping".to_string(),
            JobResultDetail::LatestBlock(_) => "LatestBlock".to_string(),
            JobResultDetail::Benchmark(_) => "Benchmark".to_string(),
            JobResultDetail::Compound(_) => "Compound".to_string(),
            JobResultDetail::Websocket(_) => "Websocket".to_string(),
        }
    }
    /*
    pub fn get_plan_id(&self) -> String {
        match self {
            JobResultDetail::HttpRequest(detail) => detail.job.plan_id.clone(),
            JobResultDetail::RpcRequest(detail) => detail.job.plan_id.clone(),
            JobResultDetail::Command(detail) => detail.job.plan_id.clone(),
            JobResultDetail::Ping(detail) => detail.job.plan_id.clone(),
            JobResultDetail::LatestBlock(detail) => detail.job.plan_id.clone(),
            JobResultDetail::Benchmark(detail) => detail.job.plan_id.clone(),
            JobResultDetail::Compound(detail) => detail.job.plan_id.clone(),
            JobResultDetail::Websocket(detail) => detail.job.plan_id.clone(),
        }
    }
     */
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Config {
    pub check_interval_ms: u64,
    pub check_task_list_node: Vec<String>,
    pub check_task_list_all: Vec<String>,
    pub check_task_list_gateway: Vec<String>,
    pub max_json_body_size: u64,
    pub response_time_key: String,
    pub max_length_report_detail: usize,
    pub benchmark_thread: i32,
    pub benchmark_connection: i32,
    pub benchmark_duration: String,
    pub benchmark_rate: i32,
    pub benchmark_script: String,
    pub benchmark_wrk_path: String,
    pub check_path_timeout_ms: u64,
    pub success_percent_threshold: u32,
    pub node_response_time_threshold_ms: f32,
    pub gateway_response_time_threshold_ms: f32,
    pub accepted_low_latency_percent: f32,
    pub skip_benchmark: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub enum JobRole {
    Verification,
    Regular,
}

impl Default for JobRole {
    fn default() -> Self {
        JobRole::Verification
    }
}

impl ToString for JobRole {
    fn to_string(&self) -> String {
        match self {
            JobRole::Verification => "verification".to_string(),
            JobRole::Regular => "regular".to_string(),
        }
    }
}
impl FromStr for JobRole {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "regular" => Ok(JobRole::Regular),
            "verification" => Ok(JobRole::Verification),
            _ => Err(anyhow::Error::msg(format!(
                "Cannot convert {} to Job role",
                s
            ))),
        }
    }
}
