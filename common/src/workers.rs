use crate::component::Zone;
use crate::jobs::Job;
use crate::models::TimeFrames;
use crate::{ComponentInfo, IPAddress, PlanId, WorkerId, COMMON_CONFIG};
use anyhow::anyhow;
use rand::Rng;
use reqwest::Body;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

#[derive(Default, Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct WorkerStatus {
    pub jobs_number_in_queue: usize,
    pub reports_number_in_queue: usize,
    pub jobs_stat: HashMap<String, Vec<String>>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, Eq, PartialEq, Hash)]
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
#[derive(Clone, Serialize, Deserialize, Debug, Default, Eq, PartialEq, Hash)]
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
                panic!("ZONE={}, please enter correct worker zone!!!", zone);
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
#[derive(Default, Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Hash)]
pub struct Worker {
    pub worker_info: WorkerInfo,
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
        //format!("https://{}/__worker/{}", self.worker_info.worker_ip, path)
        format!("{}/{}", self.worker_info.url, path)
    }
    pub fn get_host(&self) -> String {
        format!("{}.gw.mbr.massbitroute.net", self.worker_info.worker_id)
    }

    pub async fn send_jobs(&self, jobs: &Vec<Job>) -> Result<(), anyhow::Error> {
        let path = "handle_jobs";
        let body = serde_json::to_string(jobs)?;
        self.send_post_request(path, &body).await
    }

    pub async fn send_cancel_plans(&self, plans: &Vec<PlanId>) -> Result<(), anyhow::Error> {
        let path = "cancel_plans";
        let body = serde_json::to_string(plans)?;
        self.send_post_request(path, &body).await
    }

    pub async fn send_post_request(&self, path: &str, body: &str) -> Result<(), anyhow::Error> {
        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        let url = self.get_url(path);
        log::debug!(
            "Send request {path} to worker {:?} by url {:?} and body {:?}",
            &self.worker_info,
            url.as_str(),
            &body
        );
        let request_builder = client
            .post(url.as_str())
            .header("content-type", "application/json")
            .header("Host", self.get_host())
            .body(body.to_string())
            .timeout(Duration::from_millis(
                COMMON_CONFIG.default_http_request_timeout_ms,
            ));
        match request_builder.send().await {
            Ok(res) => {
                if res.status().is_success() {
                    log::debug!("Worker response: {:?}", res);
                    Ok(())
                } else {
                    Err(anyhow!(format!(
                        "Worker response error: {:?} and body: {:?}",
                        res.status(),
                        res.text().await
                    )))
                }
            }
            Err(err) => {
                log::error!("Error: {:?}", &err);
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
    pub measured_workers: Vec<Arc<Worker>>, //Workers order by round trip time
    pub remain_workers: Vec<Arc<Worker>>, //All remain workers
}

impl MatchedWorkers {
    pub fn get_nearby_worker(&self, ind: usize) -> Option<Arc<Worker>> {
        self.nearby_workers.get(ind).map(|arc| arc.clone())
    }
    pub fn get_best_worker(&self, ind: usize) -> Option<Arc<Worker>> {
        self.measured_workers.get(ind).map(|arc| arc.clone())
    }
    pub fn get_random_worker(&self) -> Option<Arc<Worker>> {
        if self.remain_workers.len() > 0 {
            let mut rng = rand::thread_rng();
            let ind = rng.gen_range(0..self.remain_workers.len());
            self.remain_workers.get(ind).map(|val| val.clone())
        } else {
            None
        }
    }
    pub fn get_all_workers(&self) -> Vec<Arc<Worker>> {
        let mut all_workers = Vec::new();
        for worker in self.measured_workers.iter() {
            all_workers.push(worker.clone());
        }
        for worker in self.remain_workers.iter() {
            all_workers.push(worker.clone());
        }
        all_workers
    }
}
