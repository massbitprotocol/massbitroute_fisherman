use crate::persistence::seaorm::plans;
use crate::persistence::seaorm::plans::Model;
use anyhow::anyhow;
use common::component::Zone;
use common::models::plan_entity::PlanStatus;
use common::models::PlanEntity;
use common::worker::WorkerInfo;
use log::error;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::{Condition, DatabaseConnection};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Default)]
pub struct PlanService {
    db: Arc<DatabaseConnection>,
}
impl PlanService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        PlanService { db }
    }
    /*
    pub async fn get_active(&self) -> Vec<WorkerInfo> {
        let mut res = Vec::new();
        if let Ok(workers) = schedulers::Entity::find()
            .filter(schedulers::Column::Active.eq(1))
            .all(self.db.as_ref())
            .await
        {
            for model in workers.iter() {
                res.push(WorkerInfo::from(model))
            }
        }
        res
    }
    */
    // pub async fn get_plans(&self, statuses: Vec<PlanStatus>) -> Option<PlanEntity> {
    //     let mut condition = Condition::any();
    //     for status in statuses {
    //         condition.add(plans::Column::Status.eq(status))
    //     }
    //
    //     match plans::Entity::find()
    //         .filter(condition)
    //         .all(self.db.as_ref())
    //         .await
    //     {
    //         Ok(plans) => Some(plans),
    //         Err(_) => None,
    //     }
    // }

    pub async fn store_scheduler(&self, entity: &PlanEntity) -> Result<Model, anyhow::Error> {
        let sched = plans::ActiveModel::from(entity);
        match sched.insert(self.db.as_ref()).await {
            Ok(res) => Ok(res),
            Err(err) => Err(anyhow!("{:?}", &err)),
        }
    }
}
impl From<&Model> for PlanEntity {
    fn from(info: &Model) -> Self {
        PlanEntity {
            id: info.id.clone(),
            provider_id: info.provider_id.clone(),
            request_time: info.request_time.clone(),
            finish_time: info.finish_time.clone(),
            result: info.result.clone(),
            message: info.message.clone(),
            status: PlanStatus::from_str(&*info.status).unwrap_or_default(),
            plan_id: info.plan_id.clone(),
            phase: info.phase.clone(),
        }
    }
}
