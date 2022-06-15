pub mod component;
pub mod job_action;
pub mod job_manage;
pub mod logger;
pub mod models;
pub mod task_spawn;
pub mod tasks;
pub mod util;
pub mod worker;
use crate::component::ComponentInfo;
use lazy_static::lazy_static;

pub type BlockChainType = String;
pub type NetworkType = String;
pub type UrlType = String;
pub type ComponentId = String;
pub type JobId = String;
pub type WorkerId = String;
pub type IPAddress = String;
pub type Node = ComponentInfo;
pub type Gateway = ComponentInfo;
pub type Timestamp = i64; //Store as second
pub use serde::{Deserialize, Serialize};
pub use serde_json::Value;
pub use std::env;
pub type PlanId = String;

lazy_static! {
    pub static ref DOMAIN: String = env::var("DOMAIN").expect("There is no env var DOMAIN");
}
