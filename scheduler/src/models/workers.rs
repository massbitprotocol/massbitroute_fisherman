use crate::persistence::ProviderMapModel;
use common::component::{ComponentInfo, Zone};
use common::workers::{MatchedWorkers, Worker, WorkerInfo};
use common::{ComponentId, WorkerId};
use log::debug;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/*
 * Todo: Rename to WorkerStorage
 */
#[derive(Default, Debug)]
pub struct WorkerInfoStorage {
    workers: Mutex<Vec<Arc<Worker>>>,
    map_worker_provider: Mutex<Vec<ProviderMapModel>>,
}

impl WorkerInfoStorage {
    pub fn new(workers: Vec<WorkerInfo>) -> Self {
        // let mut workers = Vec<Arc<Worker>>::default();
        // for worker_info in workers.into_iter() {
        //     let zone = worker_info.zone.clone();
        //     let worker = Arc::new(Worker::new(worker_info));
        //     if let Some(vec) = map.get_mut(&zone) {
        //         vec.push(worker);
        //     } else {
        //         map.insert(zone, vec![worker]);
        //     }
        // }
        WorkerInfoStorage {
            workers: Mutex::new(
                workers
                    .into_iter()
                    .map(|info| Arc::new(Worker::new(info)))
                    .collect(),
            ),
            map_worker_provider: Mutex::new(vec![]),
        }
    }
    pub async fn add_worker(&self, info: WorkerInfo) {
        let worker = Arc::new(Worker::new(info));
        self.workers.lock().await.push(worker);
        // let zone = info.zone.clone();
        // let worker = Arc::new(Worker::new(info));
        // let mut zone_workers = self.map_zone_workers.lock().await;
        // if let Some(vec) = zone_workers.get_mut(&zone) {
        //     vec.push(worker);
        // } else {
        //     zone_workers.insert(zone, vec![worker]);
        // }
    }
    // pub fn get_worker_by_zone_id(&self, zone: &Zone, worker_id: &WorkerId) -> Option<Arc<Worker>> {
    //     self.workers.get(zone).and_then(|workers| {
    //         workers
    //             .iter()
    //             .find(|w| w.get_id().as_str() == worker_id.as_str())
    //             .map(|r| r.clone())
    //     })
    // }
    pub async fn get_workers(&self) -> Vec<Arc<Worker>> {
        //debug!("Lock and clone all worker refs");
        self.workers
            .lock()
            .await
            .iter()
            .map(|worker| worker.clone())
            .collect()
    }
    pub async fn get_provider_distances(
        &self,
        provider: &ComponentInfo,
    ) -> HashMap<ComponentId, i32> {
        let mut distances = HashMap::<ComponentId, i32>::new();
        let map_worker_providers = self.map_worker_provider.lock().await;
        let provider_id = provider.id.as_str();
        map_worker_providers
            .iter()
            .filter(|provider| {
                provider.provider_id.as_str() == provider_id
                    && provider.ping_response_duration.is_some()
            })
            .for_each(|dist| {
                distances.insert(
                    dist.worker_id.clone(),
                    dist.ping_response_duration.as_ref().unwrap().clone(),
                );
            });
        distances
    }
    pub async fn set_map_worker_provider(&self, map_providers: Vec<ProviderMapModel>) {
        let mut worker_providers = self.map_worker_provider.lock().await;
        *worker_providers = map_providers;
    }
    pub async fn match_workers(
        &self,
        provider: &ComponentInfo,
    ) -> Result<MatchedWorkers, anyhow::Error> {
        let all_workers = self.get_workers().await;
        let nearby_workers = all_workers
            .iter()
            .filter(|worker| worker.worker_info.zone == provider.zone)
            .map(|worker| worker.clone())
            .collect();
        let mut distances = self.get_provider_distances(provider).await;
        let mut remain_workers = Vec::new();
        let mut measured_workers = Vec::new();
        for worker in all_workers.iter() {
            if distances.contains_key(&worker.worker_info.worker_id) {
                measured_workers.push(worker.clone());
            } else {
                remain_workers.push(worker.clone());
            }
        }
        measured_workers.sort_by(|a, b| {
            let d1 = distances.get(&a.worker_info.worker_id).unwrap();
            let d2 = distances.get(&b.worker_info.worker_id).unwrap();
            d1.partial_cmp(d2).unwrap()
        });
        Ok(MatchedWorkers {
            provider: provider.clone(),
            nearby_workers,
            measured_workers,
            remain_workers,
        })
    }
}
