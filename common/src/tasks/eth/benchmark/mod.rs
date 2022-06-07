pub mod executor;
pub mod generator;
use anyhow::{anyhow, Error};
pub use generator::*;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CallBenchmarkError {
    #[error("send error")]
    SendError(String),
    #[error("get body error")]
    GetBodyError(String),
}

impl CallBenchmarkError {
    pub fn get_message(&self) -> String {
        match self {
            CallBenchmarkError::SendError(message) | CallBenchmarkError::GetBodyError(message) => {
                message.to_string()
            }
        }
    }
    pub fn get_code(&self) -> u32 {
        match self {
            CallBenchmarkError::SendError(_) => 1u32,
            CallBenchmarkError::GetBodyError(_) => 2u32,
        }
    }
}
