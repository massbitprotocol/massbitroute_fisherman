use anyhow::Error;
use log::{debug, info};
use reqwest::Response;
use std::collections::HashMap;
use std::hash::Hash;

use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

use crate::component::{ComponentInfo, ComponentType};
use crate::job_action::CheckStep;
use crate::job_action::EndpointInfo;
use crate::tasks::command::{JobCommand, JobCommandResponse, JobCommandResult};
use crate::tasks::compound::JobCompound;
use crate::tasks::eth::CallBenchmarkError;
use crate::tasks::http_request::{JobHttpRequest, JobHttpResponse, JobHttpResult};
use crate::tasks::ping::{CallPingError, JobPingResult};
use crate::tasks::rpc_request::{JobRpcRequest, JobRpcResponse, JobRpcResult};
use crate::{BlockChainType, ComponentId, JobId, NetworkType, Timestamp};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use uuid::Uuid;

pub type Url = String;

const DEFAULT_JOB_INTERVAL: Timestamp = 1000;
const DEFAULT_JOB_TIMEOUT: Timestamp = 5000;

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
pub struct Job {
    pub job_id: JobId,
    pub job_name: String,
    pub component_id: ComponentId,
    pub priority: i32,               //Fist priority is 1
    pub expected_runtime: Timestamp, //timestamp in millisecond Default 0, job is executed if only expected_runtime <= current timestamp
    pub parallelable: bool,          //Job can be executed parallel with other jobs
    pub timeout: Timestamp,
    pub component_url: Url,
    pub repeat_number: i32,  //0-don't repeat
    pub interval: Timestamp, //
    pub header: HashMap<String, String>,
    pub job_detail: Option<JobDetail>,
}

impl From<&Job> for reqwest::Body {
    fn from(job: &Job) -> Self {
        reqwest::Body::from(serde_json::to_string(job).unwrap())
    }
}
impl Job {
    pub fn new(job_name: String, component: &ComponentInfo, job_detail: JobDetail) -> Self {
        let uuid = Uuid::new_v4();
        Job {
            job_id: uuid.to_string(),
            job_name,
            component_id: component.id.clone(),
            priority: 1,
            expected_runtime: 0,
            repeat_number: 0,
            timeout: DEFAULT_JOB_TIMEOUT,
            interval: DEFAULT_JOB_INTERVAL,
            header: Default::default(),
            job_detail: Some(job_detail),
            parallelable: false,
            component_url: "".to_string(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobCancel {
    /*
     * Time to perform ping request (if duration =-1 then perform ping without finish)
     */
    job_id: JobId,
    reason: String, //Using for log
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobPing {}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
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
    pub response_timestamp: Timestamp, //Time to get response
    pub responses: BenchmarkResponse,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum JobDetail {
    HttpRequest(JobHttpRequest),
    RpcRequest(JobRpcRequest),
    Command(JobCommand),
    // Perform some request to node/gateway
    Compound(JobCompound),
    // perform ping check
    Ping(JobPing),
    // perform benchmark checking
    Benchmark(JobBenchmark),
}

impl JobDetail {
    pub fn get_job_name(&self) -> String {
        match self {
            JobDetail::HttpRequest(_) => "HttpRequest".to_string(),
            JobDetail::RpcRequest(_) => "RpcRequest".to_string(),
            JobDetail::Command(_) => "Command".to_string(),
            JobDetail::Compound(_) => "Compound".to_string(),
            JobDetail::Ping(_) => "Ping".to_string(),
            JobDetail::Benchmark(_) => "Benchmark".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum JobResult {
    HttpRequest(JobHttpResult),
    RpcRequest(JobRpcResult),
    Command(JobCommandResult),
    // perform ping check
    Ping(JobPingResult),
    // Perform some request to node/gateway
    Compound(JobCompoundResult),
    // perform benchmark checking
    Benchmark(JobBenchmarkResult),
}

impl JobResult {
    pub fn new(job: &Job) -> Self {
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        match job.job_detail.as_ref().unwrap() {
            JobDetail::HttpRequest(rpc) => {
                JobResult::HttpRequest(JobHttpResult::new(job.clone(), JobHttpResponse::default()))
            }
            JobDetail::RpcRequest(rpc) => {
                JobResult::RpcRequest(JobRpcResult::new(job.clone(), JobRpcResponse::default()))
            }
            JobDetail::Command(rpc) => JobResult::Command(JobCommandResult::new(
                job.clone(),
                JobCommandResponse::default(),
            )),
            JobDetail::Ping(_) => JobResult::Ping(JobPingResult {
                job: job.clone(),
                //response_timestamp: current_timestamp,
                ..Default::default()
            }),
            JobDetail::Compound(_) => JobResult::Compound(JobCompoundResult {
                job: job.clone(),
                response_timestamp: current_timestamp as i64,
                ..Default::default()
            }),
            JobDetail::Benchmark(_) => JobResult::Benchmark(JobBenchmarkResult {
                job: job.clone(),
                response_timestamp: current_timestamp as i64,
                ..Default::default()
            }),
        }
    }
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

#[derive(Deserialize, Serialize, Debug, Clone)]
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
            JobRole::Regular => "fisherman".to_string(),
        }
    }
}
