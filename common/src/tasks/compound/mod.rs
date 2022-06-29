use crate::job_action::{CheckStep, EndpointInfo};
use crate::{BlockChainType, NetworkType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub mod executor;
pub mod generator;

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct JobCompound {
    pub check_steps: Vec<CheckStep>,
    pub base_endpoints: HashMap<BlockChainType, HashMap<NetworkType, Vec<EndpointInfo>>>,
}
