use crate::persistence::ProviderMapModel;
use common::component::{ComponentInfo, Zone};
use common::workers::{MatchedWorkers, Worker, WorkerInfo};
use common::{ComponentId, WorkerId};
use std::collections::HashMap;
use std::sync::Arc;

/*
 * Todo: Rename to WorkerStorage
 */
#[derive(Default, Debug)]
pub struct WorkerInfoStorage {
    map_zone_workers: HashMap<Zone, Vec<Arc<Worker>>>,
    map_worker_provider: Vec<ProviderMapModel>,
}

impl WorkerInfoStorage {
    pub fn new(workers: Vec<WorkerInfo>) -> Self {
        let mut map = HashMap::<Zone, Vec<Arc<Worker>>>::default();
        for worker_info in workers.into_iter() {
            let zone = worker_info.zone.clone();
            let worker = Arc::new(Worker::new(worker_info));
            if let Some(vec) = map.get_mut(&zone) {
                vec.push(worker);
            } else {
                map.insert(zone, vec![worker]);
            }
        }
        WorkerInfoStorage {
            map_zone_workers: map,
            map_worker_provider: vec![],
        }
    }
    pub fn add_worker(&mut self, info: WorkerInfo) {
        let zone = info.zone.clone();
        let worker = Arc::new(Worker::new(info));
        if let Some(vec) = self.map_zone_workers.get_mut(&zone) {
            vec.push(worker);
        } else {
            self.map_zone_workers.insert(zone, vec![worker]);
        }
    }
    pub fn get_worker_by_zone_id(&self, zone: &Zone, worker_id: &WorkerId) -> Option<Arc<Worker>> {
        self.map_zone_workers.get(zone).and_then(|workers| {
            workers
                .iter()
                .find(|w| w.get_id().as_str() == worker_id.as_str())
                .map(|r| r.clone())
        })
    }
    pub fn get_workers(&self, zone: &Zone) -> Option<&Vec<Arc<Worker>>> {
        self.map_zone_workers.get(zone)
    }
    pub fn match_workers(&self, provider: &ComponentInfo) -> Result<MatchedWorkers, anyhow::Error> {
        let zone_workers = self
            .map_zone_workers
            .get(&provider.zone)
            .and_then(|workers| {
                Some(
                    workers
                        .iter()
                        .map(|r| r.clone())
                        .collect::<Vec<Arc<Worker>>>(),
                )
            })
            .unwrap_or(vec![]);

        let mut distances = HashMap::<ComponentId, i32>::new();
        let provider_id = provider.id.as_str();
        for dist in self.map_worker_provider.iter() {
            if dist.provider_id.as_str() == provider_id && dist.ping_response_duration.is_some() {
                distances.insert(
                    dist.worker_id.clone(),
                    dist.ping_response_duration.as_ref().unwrap().clone(),
                );
            }
        }
        let mut remain_workers = Vec::new();
        let mut measured_workers = Vec::new();
        for (_, workers) in self.map_zone_workers.iter() {
            for w in workers {
                if distances.contains_key(&w.worker_info.worker_id) {
                    measured_workers.push(w.clone());
                } else {
                    remain_workers.push(w.clone());
                }
            }
        }
        measured_workers.sort_by(|a, b| {
            let d1 = distances.get(&a.worker_info.worker_id).unwrap();
            let d2 = distances.get(&b.worker_info.worker_id).unwrap();
            d1.partial_cmp(d2).unwrap()
        });
        Ok(MatchedWorkers {
            provider: provider.clone(),
            nearby_workers: zone_workers,
            measured_workers,
            remain_workers,
        })
    }
    pub fn set_map_worker_provider(&mut self, map_providers: Vec<ProviderMapModel>) {
        self.map_worker_provider = map_providers;
    }
}
