use std::collections::HashMap;
use std::hash::Hash;
use serde::{Serialize, Deserialize};

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
pub enum NodeType {
    NODE,
    GATEWAY
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Job {
    job_id: String,
    note_type: NodeType, //
    /*
     * Fist priority is 1
     */
    priority: u32,
    job_url: String,
    header: HashMap<String, String>,
    /*
     * For fisherman call to send job result
     */
    callback_url: String,
    /*
     *
     */
    job_detail: JobDetail,
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobCancel {
    /*
     * Time to perform ping request (if duration =-1 then perform ping without finish)
     */
    job_id: String,
    reason: String,     //Using for log
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobPing {
    duration: i32,      //Time to perform ping request (if duration =-1 then perform ping without finish)
    sleep_time: u32,    //Time beetween 2 requests
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobRequest {
    id: String,
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    /*
     * expected fields with path to get value in result
     */
    result_fields: HashMap<String, Vec<String>>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobBenchmark {
    rate: u32,              //Requests/sec
    duration: u32,          //Time to perform benchmark in ms
    timeout: u32,           //timeout foreach request
    historgrams: Vec<u32>,  //List of expected percentile,
    request: JobRequest
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobCancelResult {
    /*
     * Time to perform ping request (if duration =-1 then perform ping without finish)
     */
    job_id: String,
    status: String,
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobPingResult {
    job_id: String,
    duration: u32,
    sleep_time: u32,
    response_time: Vec<u32>, //response time or -1 if timed out
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobRequestResult {
    job_id: String,
    request_timestamp: u64,      //Time to send request
    response_timestamp: u64,     //Time to get response
    response_status: String,     //http status
    values: HashMap<String, serde_json::Value>
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobBenchmarkResult {
    job_id: String,
    duration: u32,
    request_rate: f32,
    transfer_rate: f32,
    transfer_date_unit: String, //KB or MB
    average_latency: f32,        //In ms
    histograms: HashMap<u32, f32>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum JobDetail {
    // perform ping check
    Ping(JobPing),
    // Perform some request to node/gateway
    REQUEST(JobRequest),
    // perform benchmark checking
    BENCHMARK(JobBenchmark),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum JobResult {
    // perform ping check
    Ping(JobPingResult),
    // Perform some request to node/gateway
    REQUEST(JobRequestResult),
    // perform benchmark checking
    BENCHMARK(JobBenchmarkResult),
}
