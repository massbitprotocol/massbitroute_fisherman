pub mod executor;
pub mod generator;
use crate::job_manage::Job;
use crate::Timestamp;
pub use generator::TaskLatestBlock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobLatestBlock {
    pub assigned_at: Timestamp,
    pub finished_at: Timestamp,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct JobLatestBlockResult {
    pub job: Job,
    pub worker_id: String,
    pub response: LatestBlockResponse,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct LatestBlockResponse {
    pub response_time: Timestamp,
    pub block_number: u64,
    pub block_timestamp: Timestamp,
    pub http_code: u16,
    pub error_code: u32,
    pub message: String,
}
