pub mod executor;

use crate::component::ChainInfo;
use crate::jobs::{AssignmentConfig, Job};
use crate::tasks::LoadConfig;
use crate::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct JobLatestBlock {
    pub assigned_at: Timestamp,
    pub request_body: String,
    pub chain_info: ChainInfo,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct JobLatestBlockResult {
    pub job: Job,
    pub worker_id: String,
    pub response: LatestBlockResponse,
    pub execution_timestamp: Timestamp,
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct LatestBlockConfig {
    #[serde(default)]
    pub header: HashMap<String, String>,
    #[serde(default)]
    pub latest_block_request_body: String,
    #[serde(default)]
    pub latest_block_timeout_ms: Timestamp,
    #[serde(default)]
    pub late_duration_threshold_ms: i64,
    #[serde(default)]
    pub repeat_number: i32,
    #[serde(default)]
    pub interval: Timestamp,
    #[serde(default)]
    pub assignment: Option<AssignmentConfig>,
}
impl LoadConfig<LatestBlockConfig> for LatestBlockConfig {}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct LatestBlockResponse {
    pub response_time: Timestamp,
    pub block_number: u64,
    pub block_timestamp: Timestamp, // in sec
    pub block_hash: String,
    pub http_code: u16,
    pub error_code: u32,
    pub message: String,
    pub chain_info: ChainInfo,
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

impl LatestBlockResponse {
    pub fn new_error(error_code: u32, message: &str, chain_info: ChainInfo) -> Self {
        LatestBlockResponse {
            response_time: 0,
            block_number: 0,
            http_code: 0,
            error_code,
            message: message.to_string(),
            block_timestamp: 0,
            block_hash: "".to_string(),
            chain_info,
        }
    }
}
