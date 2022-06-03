use crate::component::Zone;
use crate::models::TimeFrames;
use crate::{IPAddress, WorkerId};
use reqwest::Body;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WorkerInfo {
    /*
     * In register phase WorkerId is default (empty),
     * Scheduler issue an id for worker and return in result
     */
    pub worker_id: WorkerId,
    pub worker_ip: IPAddress,
    pub url: String,
    pub zone: Zone,
    pub worker_spec: WorkerSpec,
    pub available_time_frame: Option<TimeFrames>,
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
    pub worker_id: String,
    pub report_callback: String,
}

impl WorkerRegisterResult {
    pub fn new(worker_id: String, report_callback: String) -> Self {
        WorkerRegisterResult {
            worker_id,
            report_callback,
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WorkerStateParam {}
