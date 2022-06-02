use crate::seaorm::workers;
use crate::seaorm::workers::Model;
use common::component::Zone;
use common::worker::WorkerInfo;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use std::str::FromStr;
use std::sync::Arc;

pub struct WorkerService {
    db: Arc<DatabaseConnection>,
}
impl WorkerService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        WorkerService { db }
    }
    pub async fn get_all(&self) -> Vec<WorkerInfo> {
        let mut res = Vec::new();
        if let Ok(workers) = workers::Entity::find().all(self.db.as_ref()).await {
            for model in workers.iter() {
                res.push(WorkerInfo::from(model))
            }
        }
        res
    }
}
impl From<&workers::Model> for WorkerInfo {
    fn from(info: &Model) -> Self {
        let zone = match Zone::from_str(info.zone.as_str()) {
            Ok(zone) => zone,
            Err(err) => Zone::default(),
        };
        WorkerInfo {
            worker_id: info.worker_ip.clone(),
            worker_ip: info.worker_ip.clone(),
            url: info.url.clone(),
            zone,
            worker_spec: Default::default(),
            available_time_frame: None,
        }
    }
}
