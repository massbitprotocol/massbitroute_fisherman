use crate::persistence::services::plan_service::plans::Model;
use crate::persistence::PlanModel;
use anyhow::anyhow;
use common::job_manage::JobRole;
use common::models::plan_entity::PlanStatus;
use common::models::PlanEntity;
use common::util::get_current_time;
use common::{ComponentId, PlanId};
use entity::plans;
use log::{debug, warn};
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::{Condition, DatabaseConnection};
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Default)]
pub struct PlanService {
    db: Arc<DatabaseConnection>,
}
impl PlanService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        PlanService { db }
    }

    pub async fn get_plans(
        &self,
        phase: &Option<JobRole>,
        statuses: &Vec<PlanStatus>,
    ) -> Result<Vec<PlanEntity>, anyhow::Error> {
        let mut condition_status = Condition::any();
        for status in statuses {
            condition_status = condition_status.add(plans::Column::Status.eq(status.to_string()));
        }
        let condition = match phase {
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
    pub async fn get_active_plan_by_ids(
        &self,
        phase: &Option<JobRole>,
        ids: &HashSet<PlanId>,
    ) -> Result<Vec<PlanEntity>, anyhow::Error> {
        let mut condition_ids = Condition::any();
        for id in ids {
            condition_ids = condition_ids.add(plans::Column::PlanId.eq(id.to_string()));
        }
        let mut condition = match phase {
            None => condition_ids,
            Some(phase) => {
                let mut condition = Condition::all();
                condition = condition.add(plans::Column::Phase.eq(phase.to_string()));
                if !condition.is_empty() {
                    condition = condition.add(condition_ids);
                }
                condition
            }
        };
        let current_time = get_current_time();
        condition = condition.add(plans::Column::ExpiryTime.gt(current_time));
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
    pub async fn get_active_plan_by_components(
        &self,
        phase: &Option<JobRole>,
        providers: &HashSet<ComponentId>,
    ) -> Result<Vec<PlanEntity>, anyhow::Error> {
        let mut condition_providers = Condition::any();
        for provider in providers {
            condition_providers =
                condition_providers.add(plans::Column::ProviderId.eq(provider.to_string()));
        }
        let mut condition = match phase {
            None => condition_providers,
            Some(phase) => {
                let mut condition = Condition::all();
                condition = condition.add(plans::Column::Phase.eq(phase.to_string()));
                if !condition.is_empty() {
                    condition = condition.add(condition_providers);
                }
                condition
            }
        };
        let current_time = get_current_time();
        condition = condition.add(plans::Column::ExpiryTime.gt(current_time));
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
        let condition = match phase {
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
        debug!("Store plan {:?}", &sched);
        match sched.insert(self.db.as_ref()).await {
            Ok(res) => Ok(res),
            Err(err) => Err(anyhow!("{:?}", &err)),
        }
    }

    pub async fn store_plans(&self, vec_plans: &[PlanEntity]) -> Result<i64, anyhow::Error> {
        let records = vec_plans
            .iter()
            .map(plans::ActiveModel::from)
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
