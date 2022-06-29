use common::component::ComponentType;
use common::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct ProviderTask {
    pub provider_id: String,
    pub provider_type: ComponentType,
    pub task_type: String, //HttpRequest, Command
    pub task_name: String, //RoundTripTime, LatestBlock
}

impl ProviderTask {
    pub fn new(
        provider_id: String,
        provider_type: ComponentType,
        task_type: String,
        task_name: String,
    ) -> Self {
        Self {
            provider_id,
            provider_type,
            task_type,
            task_name,
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StoredJobResult {
    pub plan_id: String,
    pub job_id: String,
    pub worker_id: String,
    pub provider_id: String,
    pub provider_type: String,
    pub chain_id: Option<String>, //Format blockchain.network Exp eth.rinkerby
    pub detail: ResultDetail,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PingResultDetail {
    pub response_times: Vec<i64>,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LatestBlockResultDetail {
    pub response_time: Timestamp,
    pub block_number: u64,
    pub block_timestamp: Timestamp,
    pub block_hash: String,
    pub http_code: u16,
    pub error_code: u32,
    pub message: String,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BenchmarkResultDetail {
    pub connection: u32,
    pub thread: u32,
    pub rate: u32,            // Requests/sec
    pub duration: Timestamp,  // Time to perform benchmark in ms
    pub script: String,       // Name of .lua script
    pub histograms: Vec<u32>, // List of expected percentile,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ResultDetail {
    Ping(PingResultDetail),
    LatestBlock(LatestBlockResultDetail),
    Benchmark(BenchmarkResultDetail),
}
