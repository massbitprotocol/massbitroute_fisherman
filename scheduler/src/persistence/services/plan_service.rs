use crate::persistence::seaorm::plans;
use crate::persistence::seaorm::plans::Model;
use crate::persistence::PlanModel;
use anyhow::anyhow;
use common::component::Zone;
use common::job_manage::JobRole;
use common::models::plan_entity::PlanStatus;
use common::models::PlanEntity;
use common::workers::WorkerInfo;
use log::{debug, error};
use log::{info, warn};
use sea_orm::sea_query::Expr;
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
    pub async fn get_plans(
        &self,
        phase: &Option<JobRole>,
        statuses: &Vec<PlanStatus>,
    ) -> Result<Vec<PlanEntity>, anyhow::Error> {
        let mut condition_status = Condition::any();
        for status in statuses {
            condition_status = condition_status.add(plans::Column::Status.eq(status.to_string()));
        }
        let mut condition = match phase {
            None => condition_status,
            Some(phase) => {
                let mut condition = Condition::all();
                condition = condition.add(plans::Column::Phase.eq(phase.to_string()));
                if !statuses.is_empty() {
                    condition = condition.add(condition_status);
                }
                condition
            }
        };

        let mut vec = Vec::default();
        match plans::Entity::find()
            .filter(condition)
            .all(self.db.as_ref())
            .await
        {
            Ok(entities) => {
                for model in entities.iter() {
                    vec.push(PlanEntity::from(model))
                }
                Ok(vec)
            }
            Err(err) => Err(anyhow::Error::msg(format!("get_plans error: {:?}", err))),
        }
    }
    pub async fn get_plan_models(
        &self,
        phase: &Option<JobRole>,
        statuses: &Vec<PlanStatus>,
    ) -> Result<Vec<PlanModel>, anyhow::Error> {
        let mut condition_status = Condition::any();
        for status in statuses {
            condition_status = condition_status.add(plans::Column::Status.eq(status.to_string()));
        }
        let mut condition = match phase {
            None => condition_status,
            Some(phase) => {
                let mut condition = Condition::all();
                condition = condition.add(plans::Column::Phase.eq(phase.to_string()));
                if !statuses.is_empty() {
                    condition = condition.add(condition_status);
                }
                condition
            }
        };

        plans::Entity::find()
            .filter(condition)
            .all(self.db.as_ref())
            .await
            .map_err(|err| anyhow!("{:?}", err))
    }
    pub async fn get_plan_by_ids(
        &self,
        plan_ids: &Vec<String>,
    ) -> Result<Vec<PlanEntity>, anyhow::Error> {
        let mut id_conditions = Condition::any();
        for id in plan_ids {
            id_conditions = id_conditions.add(plans::Column::PlanId.eq(id.to_string()));
        }

        let mut vec = Vec::default();
        match plans::Entity::find()
            .filter(id_conditions)
            .all(self.db.as_ref())
            .await
        {
            Ok(entities) => {
                for model in entities.iter() {
                    vec.push(PlanEntity::from(model))
                }
                Ok(vec)
            }
            Err(err) => Err(anyhow::Error::msg(format!("get_plans error: {:?}", err))),
        }
    }
    pub async fn get_verification_plans(&self) -> Result<Vec<PlanEntity>, anyhow::Error> {
        match plans::Entity::find()
            .filter(plans::Column::Phase.eq(JobRole::Verification.to_string()))
            .all(self.db.as_ref())
            .await
        {
            Ok(entities) => {
                let mut vec = Vec::default();
                for model in entities.iter() {
                    vec.push(PlanEntity::from(model))
                }
                Ok(vec)
            }
            Err(err) => Err(anyhow::Error::msg(format!(
                "get_verification_plans error:{}",
                err
            ))),
        }
    }
    pub async fn store_plan(&self, entity: &PlanEntity) -> Result<Model, anyhow::Error> {
        let sched = plans::ActiveModel::from(entity);
        match sched.insert(self.db.as_ref()).await {
            Ok(res) => Ok(res),
            Err(err) => Err(anyhow!("{:?}", &err)),
        }
    }

    pub async fn store_plans(&self, vec_plans: &Vec<PlanEntity>) -> Result<i64, anyhow::Error> {
        let records = vec_plans
            .iter()
            .map(|plan| plans::ActiveModel::from(plan))
            .collect::<Vec<plans::ActiveModel>>();
        let length = records.len();
        warn!("save_plans records:{:?}", records);

        match plans::Entity::insert_many(records)
            .exec(self.db.as_ref())
            .await
        {
            Ok(res) => {
                log::warn!("Insert plans many records {:?}", length);
                Ok(res.last_insert_id)
            }
            Err(err) => {
                log::warn!("Error store_plans {:?}", &err);
                Err(anyhow!("{:?}", &err))
            }
        }
    }

    pub async fn update_plans_as_generated(
        &self,
        plan_ids: Vec<String>,
    ) -> Result<(), anyhow::Error> {
        let mut condition = Condition::any();
        for id in plan_ids {
            condition = condition.add(plans::Column::PlanId.eq(id.to_owned()));
        }
        match plans::Entity::update_many()
            .col_expr(
                plans::Column::Status,
                Expr::value(PlanStatus::Generated.to_string()),
            )
            .filter(condition)
            .exec(self.db.as_ref())
            .await
        {
            Ok(res) => {
                log::debug!("Update with result {:?}", &res);
                Ok(())
            }
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
