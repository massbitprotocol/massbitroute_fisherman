use crate::component::Zone;
use crate::models::TimeFrames;
use crate::{IPAddress, WorkerId};
use reqwest::Body;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WorkerInfo {
    /*
     * In register phase WorkerId is default (empty),
     * Scheduler issue an id for worker and return in result
     */
    worker_id: WorkerId,
    worker_ip: IPAddress,
    url: String,
    pub zone: Zone,
    worker_spec: WorkerSpec,
    available_time_frame: Option<TimeFrames>,
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WorkerSpec {
    cpus: u16,      //Number of cpus
    ram: u32,       //Ram capacity in Mb
    bandwidth: u32, //Bandwidth in megabytes/sec,
}
impl WorkerInfo {
    pub fn new() -> Self {
        WorkerInfo {
            worker_id: "".to_string(),
            worker_ip: "".to_string(),
            url: "".to_string(),
            zone: Default::default(),
            worker_spec: WorkerSpec::default(),
            available_time_frame: None,
        }
    }
}

impl Into<Body> for WorkerInfo {
    fn into(self) -> Body {
        todo!()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WorkerRegisterResult {
    worker_id: String,
    result: String,
}

impl WorkerRegisterResult {
    pub fn new() -> Self {
        WorkerRegisterResult {
            worker_id: "".to_string(),
            result: "".to_string(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WorkerStateParam {}
