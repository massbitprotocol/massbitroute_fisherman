use anyhow::Error;
use log::{debug, info};
use reqwest::Response;
use std::collections::HashMap;
use std::hash::Hash;

use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

use crate::component::ComponentInfo;
use crate::job_action::CheckStep;
use crate::JobId;
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
    component_info: ComponentInfo,
    priority: u32, //Fist priority is 1
    time_out: Timestamp,
    start_deadline: Timestamp,
    component_url: Url,
    repeat_number: i32,
    interval: u32,
    header: HashMap<String, String>,
    callback_url: Url, //For fisherman call to send job result
    pub job_detail: Option<JobDetail>,
}

impl From<&Job> for reqwest::Body {
    fn from(job: &Job) -> Self {
        reqwest::Body::from(serde_json::to_string(job).unwrap())
    }
}
impl Job {
    pub fn new(job_detail: JobDetail) -> Self {
        Job {
            job_id: "".to_string(),
            component_info: Default::default(),
            priority: 0,
            time_out: 0,
            start_deadline: 0,
            component_url: "".to_string(),
            repeat_number: 0,
            interval: 0,
            header: Default::default(),
            callback_url: "".to_string(),
            job_detail: Some(job_detail),
        }
    }
}
impl Job {
    pub async fn process(&self) -> JoinHandle<Result<JobResult, Error>> {
        // let task = tokio::spawn(async move {
        //     let job_detail = self.job_detail.as_ref().unwrap();
        //     match job_detail {
        //         JobDetail::Ping(job_detail) => self.process_ping().await,
        //         JobDetail::Compound(job_detail) => Ok(JobResult::new(self)),
        //         JobDetail::Benchmark(job_detail) => Ok(JobResult::new(self)),
        //     }
        // });
        // task
        todo!()
    }

    async fn process_ping(&self) -> Result<JobResult, Error> {
        for repeat_time in 1..self.repeat_number {
            info!("*** Do ping ***");
            sleep(Duration::from_millis(1000)).await;
        }

        Ok(JobResult::Ping(JobPingResult::default()))
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
    check_steps: Vec<CheckStep>,
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
    job: Job,
    response_timestamp: Timestamp, //Time to get response
    response_time: Vec<u32>,       //response time or -1 if timed out
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
