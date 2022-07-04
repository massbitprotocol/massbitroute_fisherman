use anyhow::anyhow;
use common::component::Zone;
use common::workers::WorkerInfo;
use entity::workers::Model;
use entity::{plans, workers};
use log::error;
use sea_orm::DatabaseConnection;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Default)]
pub struct WorkerService {
    db: Arc<DatabaseConnection>,
}
impl WorkerService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        WorkerService { db }
    }
    pub async fn get_active(&self) -> Vec<WorkerInfo> {
        let mut res = Vec::new();
        if let Ok(workers) = workers::Entity::find()
            .filter(workers::Column::Active.eq(1))
            .all(self.db.as_ref())
            .await
        {
            for model in workers.iter() {
                res.push(WorkerInfo::from(model))
            }
        }
        res
    }
    pub async fn get_stored_worker(&self, worker_id: &str) -> Option<WorkerInfo> {
        match workers::Entity::find()
            .filter(workers::Column::WorkerId.eq(worker_id))
            .all(self.db.as_ref())
            .await
        {
            Ok(workers) => {
                if workers.len() > 0 {
                    workers.get(0).map(|val| WorkerInfo::from(val))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
    pub async fn store_worker(&self, worker: &WorkerInfo) -> Result<workers::Model, anyhow::Error> {
        let worker = workers::ActiveModel::from(worker);
        match worker.insert(self.db.as_ref()).await {
            Ok(res) => Ok(res),
            Err(err) => Err(anyhow!("{:?}", &err)),
        }
    }
}
