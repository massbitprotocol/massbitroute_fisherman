use crate::persistence::ProviderMapModel;
use common::component::ComponentInfo;
use common::workers::{MatchedWorkers, Worker, WorkerInfo};
use common::{ComponentId, WorkerId};
use log::{debug, info};
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
        let mut workers = self.workers.lock().await;
        if !workers
            .iter()
            .any(|worker| worker.worker_info.worker_id == info.worker_id)
        {
            workers.push(Arc::new(Worker::new(info)));
        }
    }
    pub async fn remove_workers(&self, worker_ids: &[&WorkerId]) {
        let mut workers = self.workers.lock().await;
        info!(
            "Workers list before remove: {:?}, worker_ids:{:?}",
            workers, worker_ids
        );
        workers.retain(|worker| !worker_ids.contains(&&worker.worker_info.worker_id));
        info!("Workers list after remove: {:?}", workers);
    }

    pub async fn get_workers(&self) -> Vec<Arc<Worker>> {
        //debug!("Lock and clone all worker refs");
        self.workers
            .lock()
            .await
            .iter()
            .map(|worker| worker.clone())
            .collect()
    }
    pub async fn get_workers_number(&self) -> usize {
        self.workers.lock().await.len()
    }

    pub async fn get_worker(&self, worker_id: WorkerId) -> Option<Arc<Worker>> {
        //debug!("Lock and clone all worker refs");
        self.workers
            .lock()
            .await
            .iter()
            .find(|&worker| worker.worker_info.worker_id == worker_id)
            .cloned()
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
        let distances = self.get_provider_distances(provider).await;
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
        let matched_workers = MatchedWorkers {
            provider: provider.clone(),
            nearby_workers,
            measured_workers,
            remain_workers,
        };
        debug!(
            "matched workers for provider {:?} {:?}",
            provider, &matched_workers
        );
        Ok(matched_workers)
    }
}
