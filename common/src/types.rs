pub use crate::component::BlockChainType;

pub type Url = String;
//pub type BlockChainType = String;
pub type NetworkType = String;
pub type UrlType = String;
pub type ComponentId = String;
pub type JobId = String;
pub type WorkerId = String;
pub type IPAddress = String;
pub type Node = ComponentInfo;
pub type Gateway = ComponentInfo;
pub type Timestamp = i64; //Store as second
use crate::ComponentInfo;
pub use serde::{Deserialize, Serialize};
pub use serde_json::Value;
pub use std::env;

pub type PlanId = String;
