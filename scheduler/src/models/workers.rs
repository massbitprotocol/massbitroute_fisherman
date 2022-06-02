use anyhow::anyhow;
use common::component::Zone;
use common::job_manage::Job;
use common::worker::WorkerInfo;
use common::WorkerId;
use reqwest::{Error, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::format;
use std::sync::Arc;
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct WorkerInfoStorage {
    map_zone_workers: HashMap<Zone, Vec<Arc<WorkerInfo>>>,
}

impl WorkerInfoStorage {
    pub fn new(workers: Vec<WorkerInfo>) -> Self {
        let mut map = HashMap::<Zone, Vec<Arc<WorkerInfo>>>::default();
        for worker in workers.into_iter() {
            if let Some(vec) = map.get_mut(&worker.zone) {
                vec.push(Arc::new(worker));
            } else {
                map.insert(worker.zone.clone(), vec![Arc::new(worker)]);
            }
        }
        WorkerInfoStorage {
            map_zone_workers: map,
        }
    }
    pub fn add_worker(&mut self, info: WorkerInfo) {
        let vec_workers = self.map_zone_workers.get_mut(&info.zone);
        match vec_workers {
            None => {
                let mut vec = Vec::default();
                let key = info.zone.clone();
                vec.push(Arc::new(info));
                self.map_zone_workers.insert(key, vec);
            }
            Some(vec) => {
                vec.push(Arc::new(info));
            }
        }
    }
    pub fn get_worker_by_zone_id(
        &self,
        zone: &Zone,
        worker_id: &WorkerId,
    ) -> Option<Arc<WorkerInfo>> {
        self.map_zone_workers.get(zone).and_then(|workers| {
            workers
                .iter()
                .find(|w| w.worker_id.eq(worker_id))
                .map(|r| r.clone())
        })
    }
    pub fn get_workers(&self, zone: &Zone) -> Option<&Vec<Arc<WorkerInfo>>> {
        self.map_zone_workers.get(zone)
    }
}
pub struct Worker {
    worker_info: Arc<WorkerInfo>,
}

impl Worker {
    pub fn new(info: Arc<WorkerInfo>) -> Worker {
        Worker { worker_info: info }
    }
    pub fn get_zone(&self) -> Zone {
        self.worker_info.zone.clone()
    }
    pub fn has_id(&self, id: &WorkerId) -> bool {
        self.worker_info.worker_id.eq(id)
    }
    pub async fn send_job(&self, job: &Job) -> Result<(), anyhow::Error> {
        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        log::debug!("Send job {:?} to worker {:?}", &job, &self.worker_info);
        let request_builder = client
            .post(self.worker_info.url.as_str())
            .header("content-type", "application/json")
            .body(job);
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
        log::debug!("Send {} jobs to worker {:?}", jobs.len(), &self.worker_info);
        if let Ok(body) = serde_json::to_string(jobs) {}
        let request_builder = client
            .post(self.worker_info.url.as_str())
            .header("content-type", "application/json")
            .body(serde_json::to_string(jobs)?);
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
