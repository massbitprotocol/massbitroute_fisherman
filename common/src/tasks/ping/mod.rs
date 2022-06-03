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
    pub fn get_message(err: &CallPingError) -> String {
        match err {
            CallPingError::BuildError(message)
            | CallPingError::SendError(message)
            | CallPingError::GetBodyError(message) => message.to_string(),
        }
    }
    pub fn get_code(err: &CallPingError) -> u32 {
        err as u32
    }
}
