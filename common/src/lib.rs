pub mod component;
pub mod job_action;
pub mod job_manage;
pub mod logger;
pub mod models;
pub mod task_spawn;
pub mod worker;

use crate::component::ComponentInfo;

pub type BlockChainType = String;
pub type NetworkType = String;
pub type UrlType = String;
pub type ComponentId = String;
pub type JobId = String;
pub type WorkerId = String;
pub type IPAddress = String;
pub type Node = ComponentInfo;
pub type Gateway = ComponentInfo;
pub type Timestamp = u128;

pub use serde::{Deserialize, Serialize};
pub use serde_json::Value;
