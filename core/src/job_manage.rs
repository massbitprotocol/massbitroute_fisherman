use std::collections::HashMap;
use std::hash::Hash;

use serde::{Serialize, Deserialize};
use crate::component::ComponentInfo;
use crate::job_action::CheckStep;

type Url = String;
type JobId = String;
type Timestamp = u64;

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
    GATEWAY
}
#[derive(Clone, Serialize, Deserialize, Debug,Default)]
pub struct Job {
    job_id: JobId,
    component_info: ComponentInfo, //
    /*
     * Fist priority is 1
     */
    priority: u32,
    time_out:Timestamp,
    start_deadline: Timestamp,
    component_url: Url,
    repeat_number: i32,
    interval: u32,
    header: HashMap<String, String>,
    /*
     * For fisherman call to send job result
     */
    callback_url: Url,
    /*
     *
     */
    job_detail: Option<JobDetail>,
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobCancel {
    /*
     * Time to perform ping request (if duration =-1 then perform ping without finish)
     */
    job_id: JobId,
    reason: String,     //Using for log
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
    rate: u32,              // Requests/sec
    duration: u64,          // Time to perform benchmark in ms
    timeout: Timestamp,           // Timeout foreach request
    script: String,         // Name of .lua script
    histograms: Vec<u32>,   // List of expected percentile,
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
    request_timestamp: u64,      //Time to send request
    response_timestamp: u64,     //Time to get response
    response_time: Vec<u32>, //response time or -1 if timed out

}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobCompoundResult {
    job: Job,
    request_timestamp: u64,      //Time to send request
    response_timestamp: u64,     //Time to get response
    response_status: String,     //http status
    values: HashMap<String, serde_json::Value>
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobBenchmarkResult {
    job: Job,
    request_timestamp: u64,      //Time to send request
    response_timestamp: u64,     //Time to get response
    request_rate: f32,
    transfer_rate: f32,          //KB
    average_latency: f32,        //In ms
    histograms: HashMap<u32, f32>
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
