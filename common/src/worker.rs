use crate::component::Zone;
use crate::models::TimeFrames;
use crate::{IPAddress, WorkerId};
use reqwest::Body;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

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
    pub fn new(worker_id: &str, worker_endpoint: &str, worker_ip: &str, zone: &str) -> Self {
        let zone = match Zone::from_str(zone) {
            Ok(zone) => zone,
            Err(_) => {
                panic!("Please enter worker zone!!!");
            }
        };
        WorkerInfo {
            worker_id: String::from(worker_id),
            worker_ip: String::from(worker_ip),
            url: String::from(worker_endpoint),
            zone,
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
