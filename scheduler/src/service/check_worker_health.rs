use crate::models::job_result_cache::JobResultCache;
use crate::models::workers::WorkerInfoStorage;

use crate::server_builder::SimpleResponse;
use crate::CONFIG;
use anyhow::{anyhow, Error};
use common::util::get_current_time;
use common::workers::Worker;
use common::{Timestamp, WorkerId, DEFAULT_HTTP_REQUEST_TIMEOUT};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

struct WorkerStatus {
    worker: Worker,
    health: WorkerHealth,
    update_time: Timestamp,
}

impl WorkerStatus {
    pub fn new(worker: &Worker, health: WorkerHealth) -> WorkerStatus {
        WorkerStatus {
            worker: worker.clone(),
            health,
            update_time: get_current_time(),
        }
    }
}

#[derive(PartialEq)]
enum WorkerHealth {
    Good,
    Bad,
}

#[derive(Default)]
pub struct WorkerHealthService {
    workers: Arc<WorkerInfoStorage>,
    result_cache: Arc<JobResultCache>,
    workers_status: HashMap<WorkerId, WorkerStatus>,
}

impl WorkerHealthService {
    pub fn new(workers: Arc<WorkerInfoStorage>, result_cache: Arc<JobResultCache>) -> Self {
        WorkerHealthService {
            workers,
            result_cache,
            workers_status: HashMap::new(),
        }
    }
    pub async fn run(mut self) {
        loop {
            let workers = self.workers.get_workers().await;
            info!("Get {} workers from list", workers.len());
            for worker in workers {
                let worker_status = self
                    .workers_status
                    .entry(worker.worker_info.worker_id.to_string())
                    .or_insert_with(|| WorkerStatus::new(&*worker, WorkerHealth::Good));
                // If worker exist in self.workers list and it heal is bad -> it have been restart.
                if worker_status.health == WorkerHealth::Bad {
                    worker_status.health = WorkerHealth::Good;
                    worker_status.update_time = get_current_time();
                }
            }

            self.update_status_and_remove_bad_worker().await;

            info!("Sleep for {} seconds", CONFIG.update_provider_list_interval);
            sleep(Duration::from_secs(
                CONFIG.update_worker_list_interval as u64,
            ))
            .await;
        }
    }

    async fn update_status_and_remove_bad_worker(&mut self) {
        {
            let results = self.result_cache.result_cache_map.lock().await;
            // Check if worker sent results in cache
            for results_component in results.values() {
                for results_task in results_component.values() {
                    for result in results_task.results.iter() {
                        if let Some(worker) = self.workers_status.get_mut(&*result.worker_id) {
                            if worker.update_time < result.receive_timestamp {
                                //info!("Update worker status: worker.update_time: {} < result.receive_timestamp: {}",worker.update_time, result.receive_timestamp);
                                worker.update_time = result.receive_timestamp;
                                worker.health = WorkerHealth::Good;
                            }
                        }
                    }
                }
            }
        }

        for (_id, status) in self.workers_status.iter_mut() {
            let now = get_current_time();
            if now - status.update_time > CONFIG.update_worker_list_interval * 1000 {
                let res = Self::ping_worker(&*status.worker.worker_info.url).await;
                status.update_time = now;
                if res.is_err() {
                    if status.health == WorkerHealth::Good {
                        // Remove cannot ping worker
                        warn!(
                            "Remove worker {} {} from working list, err {:?}.",
                            status.worker.worker_info.worker_id, status.worker.worker_info.url, res
                        );
                        self.workers
                            .remove_workers(&[&status.worker.worker_info.worker_id])
                            .await;
                    }
                    status.health = WorkerHealth::Bad;
                } else {
                    if status.health == WorkerHealth::Bad {
                        // Add good heal worker
                        warn!(
                            "Ping ok, add worker {} {} from working list.",
                            status.worker.worker_info.worker_id, status.worker.worker_info.url
                        );
                        self.workers
                            .add_worker(status.worker.worker_info.clone())
                            .await;
                    }
                    status.health = WorkerHealth::Good;
                }
            } else {
                if status.health == WorkerHealth::Bad {
                    // Add good heal worker
                    warn!(
                        "Update new result, add worker {} {} from working list.",
                        status.worker.worker_info.worker_id, status.worker.worker_info.url
                    );
                    self.workers
                        .add_worker(status.worker.worker_info.clone())
                        .await;
                }
                status.health = WorkerHealth::Good;
            }
        }
    }
    async fn ping_worker(worker_url: &str) -> Result<(), Error> {
        let url = format!("{}/ping", worker_url);
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_millis(4000))
            .build()?;
        let resp = client
            .get(&url)
            .timeout(Duration::from_millis(DEFAULT_HTTP_REQUEST_TIMEOUT))
            .send()
            .await?
            .text()
            .await?;
        let resp: SimpleResponse = serde_json::from_str(&resp)?;
        let expect_resp = SimpleResponse { success: true };
        debug!("ping {} worker rtt: {:?}", url, resp);
        if resp != expect_resp {
            return Err(anyhow!("Worker wrong response: {:?}", expect_resp));
        }
        Ok(())
    }
}
