use common::component::Zone;
use common::worker::WorkerInfo;
use std::collections::HashMap;

#[derive(Default)]
pub struct WorkerPool {
    map_zone_workers: HashMap<Zone, Vec<WorkerInfo>>,
}

impl WorkerPool {
    pub fn new() -> Self {
        WorkerPool {
            map_zone_workers: Default::default(),
        }
    }
    pub fn add_worker(&mut self, worker: WorkerInfo) {
        let vec_workers = self.map_zone_workers.get_mut(&worker.zone);
        match vec_workers {
            None => {
                let mut vec = Vec::default();
                let key = worker.zone.clone();
                vec.push(worker);
                self.map_zone_workers.insert(key, vec);
            }
            Some(vec) => {
                vec.push(worker);
            }
        }
    }
}
