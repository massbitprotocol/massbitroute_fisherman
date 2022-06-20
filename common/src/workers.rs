use crate::component::Zone;
use crate::jobs::Job;
use crate::models::TimeFrames;
use crate::{ComponentInfo, IPAddress, WorkerId};
use anyhow::anyhow;
use reqwest::Body;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

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
    bandwidth: u32, //Bandwidth in megabits/sec,
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
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Worker {
    worker_info: WorkerInfo,
}

impl Worker {
    pub fn new(info: WorkerInfo) -> Worker {
        Worker { worker_info: info }
    }
    pub fn get_id(&self) -> WorkerId {
        self.worker_info.worker_id.clone()
    }
    pub fn get_info(&self) -> WorkerInfo {
        self.worker_info.clone()
    }
    pub fn get_zone(&self) -> Zone {
        self.worker_info.zone.clone()
    }
    pub fn has_id(&self, id: &WorkerId) -> bool {
        self.worker_info.worker_id.eq(id)
    }
    pub fn get_url(&self, path: &str) -> String {
        format!("https://{}/__worker/{}", self.worker_info.worker_ip, path)
        //format!("{}/{}", self.worker_info.url, path)
    }
    pub fn get_host(&self) -> String {
        let parts = self.worker_info.url.split('/').collect::<Vec<&str>>();
        if parts.len() >= 3 {
            parts[2].to_string()
        } else {
            String::default()
        }
        //format!("{}/{}", self.worker_info.url, path)
    }
    pub async fn send_job(&self, job: &Job) -> Result<(), anyhow::Error> {
        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        let url = self.get_url("jobs_handle");
        log::debug!(
            "Send 1 jobs to worker {:?} by url {:?}",
            &self.worker_info,
            url.as_str()
        );
        let request_builder = client
            .post(self.worker_info.url.as_str())
            .header("content-type", "application/json")
            .body(serde_json::to_string(&vec![job])?);
        match request_builder.send().await {
            Ok(res) => {
                log::debug!("Worker response: {:?}", res);
                Ok(())
            }
            Err(err) => {
                log::debug!("Error:{:?}", &err);
                Err(anyhow!(format!("{:?}", &err)))
            }
        }
    }
    pub async fn send_jobs(&self, jobs: &Vec<Job>) -> Result<(), anyhow::Error> {
        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        let url = self.get_url("jobs_handle");
        let body = serde_json::to_string(jobs)?;
        log::debug!(
            "Send {} jobs to worker {:?} by url {:?} and body {:?}",
            jobs.len(),
            &self.worker_info,
            url.as_str(),
            &body
        );
        let request_builder = client
            .post(url.as_str())
            .header("content-type", "application/json")
            .header("Host", self.get_host())
            .body(body);
        match request_builder.send().await {
            Ok(res) => {
                log::debug!("Worker response: {:?}", res);
                Ok(())
            }
            Err(err) => {
                log::debug!("Error:{:?}", &err);
                Err(anyhow!(format!("{:?}", &err)))
            }
        }
    }
}
/*
 */
#[derive(Default, Debug)]
pub struct MatchedWorkers {
    pub provider: ComponentInfo,
    pub nearby_workers: Vec<Arc<Worker>>, //Workers defined by zone
    pub best_workers: Vec<Arc<Worker>>,   //Workers order by ping time
}
