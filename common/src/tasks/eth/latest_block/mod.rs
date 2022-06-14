pub mod executor;
pub mod generator;
use crate::job_manage::Job;
use crate::Timestamp;
pub use generator::LatestBlockGenerator;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobLatestBlock {
    pub assigned_at: Timestamp,
    pub request_body: String,
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
    pub block_hash: String,
    pub http_code: u16,
    pub error_code: u32,
    pub message: String,
}

#[derive(Error, Debug, Clone)]
pub enum CallLatestBlockError {
    #[error("build error")]
    BuildError(String),
    #[error("send error")]
    SendError(String),
    #[error("get body error")]
    GetBodyError(String),
}

impl CallLatestBlockError {
    pub fn get_message(&self) -> String {
        match self {
            CallLatestBlockError::BuildError(message)
            | CallLatestBlockError::SendError(message)
            | CallLatestBlockError::GetBodyError(message) => message.to_string(),
        }
    }
    pub fn get_code(&self) -> u32 {
        match self {
            CallLatestBlockError::BuildError(_) => 1u32,
            CallLatestBlockError::SendError(_) => 2u32,
            CallLatestBlockError::GetBodyError(_) => 3u32,
        }
    }
}

impl From<CallLatestBlockError> for LatestBlockResponse {
    fn from(error: CallLatestBlockError) -> Self {
        LatestBlockResponse::new_error(error.get_code(), error.get_message().as_str())
    }
}

impl LatestBlockResponse {
    pub fn new_error(error_code: u32, message: &str) -> Self {
        LatestBlockResponse {
            response_time: 0,
            block_number: 0,
            http_code: 0,
            error_code,
            message: message.to_string(),
            block_timestamp: 0,
            block_hash: "".to_string(),
        }
    }
}
