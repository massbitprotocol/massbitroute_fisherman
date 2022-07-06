pub mod executor;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CallBenchmarkError {
    #[error("get job info error")]
    GetJobInfoError(String),
    #[error("send error")]
    SendError(String),
    #[error("get body error")]
    ParseResultError(String),
}

impl CallBenchmarkError {
    pub fn get_message(&self) -> String {
        match self {
            CallBenchmarkError::GetJobInfoError(message)
            | CallBenchmarkError::SendError(message)
            | CallBenchmarkError::ParseResultError(message) => message.to_string(),
        }
    }
    pub fn get_code(&self) -> u32 {
        match self {
            CallBenchmarkError::GetJobInfoError(_) => 1u32,
            CallBenchmarkError::SendError(_) => 2u32,
            CallBenchmarkError::ParseResultError(_) => 3u32,
        }
    }
}
