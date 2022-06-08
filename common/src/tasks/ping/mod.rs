pub mod executor;
pub mod generator;
use crate::job_manage::Job;
use crate::Timestamp;
use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CallPingError {
    #[error("build error")]
    BuildError(String),
    #[error("send error")]
    SendError(String),
    #[error("get body error")]
    GetBodyError(String),
}

impl CallPingError {
    pub fn get_message(&self) -> String {
        match self {
            CallPingError::BuildError(message)
            | CallPingError::SendError(message)
            | CallPingError::GetBodyError(message) => message.to_string(),
        }
    }
    pub fn get_code(&self) -> u32 {
        match self {
            CallPingError::BuildError(_) => 1u32,
            CallPingError::SendError(_) => 2u32,
            CallPingError::GetBodyError(_) => 3u32,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobPingResult {
    pub job: Job,
    //pub response_timestamp: Timestamp, //Time to get response
    pub response: PingResponse,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct PingResponse {
    pub response_time: Timestamp,
    pub response_body: String,
    pub http_code: u16,
    pub error_code: u32,
    pub message: String,
}

impl From<CallPingError> for PingResponse {
    fn from(error: CallPingError) -> Self {
        PingResponse::new_error(error.get_code(), error.get_message().as_str())
    }
}

impl PingResponse {
    pub fn new_error(error_code: u32, message: &str) -> Self {
        PingResponse {
            response_time: 0,
            response_body: "".to_string(),
            http_code: 0,
            error_code,
            message: message.to_string(),
        }
    }
}
