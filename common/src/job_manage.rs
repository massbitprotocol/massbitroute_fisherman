use anyhow::Error;
use log::{debug, info};
use reqwest::Response;
use std::collections::HashMap;
use std::hash::Hash;

use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

use crate::component::ComponentInfo;
use crate::job_action::CheckStep;
use crate::job_action::EndpointInfo;
use crate::{BlockChainType, JobId, NetworkType};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

type Url = String;
type Timestamp = u128;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, Hash, Eq)]
pub enum JobType {
    // perform ping check
    PING,
    // Perform some request to node/gateway
    REQUEST,
    // perform benchmark checking
    BENCHMARK,
}
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, Hash, Eq)]
pub enum ComponentType {
    NODE,
    GATEWAY,
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Job {
    pub job_id: JobId,
    pub component_info: ComponentInfo,
    pub priority: u32, //Fist priority is 1
    pub time_out: Timestamp,
    pub start_deadline: Timestamp,
    pub component_url: Url,
    pub repeat_number: i32,
    pub interval: u32,
    pub header: HashMap<String, String>,
    pub callback_url: Url, //For fisherman call to send job result
    pub job_detail: Option<JobDetail>,
    pub running_mode: JobRunningMode,
    pub config: Option<Config>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum JobRunningMode {
    Endless(EndlessMode),
    Finite(FiniteMode),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FiniteMode {}

impl Default for JobRunningMode {
    fn default() -> Self {
        JobRunningMode::Finite { 0: FiniteMode {} }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EndlessMode {
    report_interval: u32, // Number of results before report
}

impl From<&Job> for reqwest::Body {
    fn from(job: &Job) -> Self {
        reqwest::Body::from(serde_json::to_string(job).unwrap())
    }
}
impl Job {
    pub fn new(job_detail: JobDetail) -> Self {
        Job {
            job_detail: Some(job_detail),
            ..Default::default()
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
pub struct JobCompound {
    pub check_steps: Vec<CheckStep>,
    pub base_endpoints: HashMap<BlockChainType, HashMap<NetworkType, Vec<EndpointInfo>>>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobBenchmark {
    connection: u32,
    thread: u32,
    rate: u32,            // Requests/sec
    duration: Timestamp,  // Time to perform benchmark in ms
    timeout: Timestamp,   // Timeout foreach request
    script: String,       // Name of .lua script
    histograms: Vec<u32>, // List of expected percentile,
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
pub struct JobPingResult {
    pub job: Job,
    pub response_timestamp: Timestamp, //Time to get response
    pub response_time: Vec<u32>,       //response time or -1 if timed out
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
    job: Job,
    response_timestamp: Timestamp, //Time to get response
    request_rate: f32,
    transfer_rate: f32,   //KB
    average_latency: f32, //In ms
    histograms: HashMap<u32, f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum JobDetail {
    // perform ping check
    Ping(JobPing),
    // Perform some request to node/gateway
    Compound(JobCompound),
    // perform benchmark checking
    Benchmark(JobBenchmark),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum JobResult {
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
            JobDetail::Ping(_) => JobResult::Ping(JobPingResult {
                job: job.clone(),
                response_timestamp: current_timestamp,
                ..Default::default()
            }),
            JobDetail::Compound(_) => JobResult::Compound(JobCompoundResult {
                job: job.clone(),
                response_timestamp: current_timestamp,
                ..Default::default()
            }),
            JobDetail::Benchmark(_) => JobResult::Benchmark(JobBenchmarkResult {
                job: job.clone(),
                response_timestamp: current_timestamp,
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
