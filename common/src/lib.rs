pub mod component;
pub mod job_action;
pub mod job_manage;
pub mod logger;
pub mod models;
pub mod worker;

pub type BlockChainType = String;
pub type NetworkType = String;
pub type UrlType = String;
pub type ComponentId = String;
pub type JobId = String;

pub use serde::{Deserialize, Serialize};
pub use serde_json::Value;
