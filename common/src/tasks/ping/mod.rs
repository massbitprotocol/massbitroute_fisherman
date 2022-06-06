pub mod executor;
pub mod generator;
use anyhow::{anyhow, Error};
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
