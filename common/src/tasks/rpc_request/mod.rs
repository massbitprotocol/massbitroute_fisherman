use crate::jobs::Job;
use crate::{ComponentInfo, Timestamp};
use serde::{Deserialize, Serialize};
use thiserror::Error;
pub mod executor;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct JobRpcRequest {}

impl JobRpcRequest {
    pub fn get_component_url(&self, _component: &ComponentInfo) -> String {
        String::new()
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobRpcResponse {
    pub response_duration: Timestamp,
    pub response_body: String,
    pub http_code: u16,
    pub error_code: u32,
    pub message: String,
}

impl JobRpcResponse {
    pub fn new_error(error_code: u32, message: &str) -> Self {
        JobRpcResponse {
            response_duration: 0,
            response_body: "".to_string(),
            http_code: 0,
            error_code,
            message: message.to_string(),
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobRpcResult {
    pub job: Job,
    pub response: JobRpcResponse,
}

impl JobRpcResult {
    pub fn new(job: Job, response: JobRpcResponse) -> Self {
        JobRpcResult { job, response }
    }
}

#[derive(Error, Debug, Clone)]
pub enum RpcRequestError {
    #[error("build error")]
    BuildError(String),
    #[error("send error")]
    SendError(String),
    #[error("get body error")]
    GetBodyError(String),
}

impl RpcRequestError {
    pub fn get_message(&self) -> String {
        match self {
            RpcRequestError::BuildError(message)
            | RpcRequestError::SendError(message)
            | RpcRequestError::GetBodyError(message) => message.to_string(),
        }
    }
    pub fn get_code(&self) -> u32 {
        match self {
            RpcRequestError::BuildError(_) => 1u32,
            RpcRequestError::SendError(_) => 2u32,
            RpcRequestError::GetBodyError(_) => 3u32,
        }
    }
}

impl From<RpcRequestError> for JobRpcResponse {
    fn from(error: RpcRequestError) -> Self {
        JobRpcResponse::new_error(error.get_code(), error.get_message().as_str())
    }
}
