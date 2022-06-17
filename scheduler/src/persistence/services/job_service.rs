use crate::persistence::seaorm::jobs;
use crate::persistence::seaorm::jobs::Model;
use anyhow::anyhow;
use common::component::Zone;
use common::jobs::Job;
use common::workers::WorkerInfo;
use log::{debug, error, log};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::{Condition, DatabaseConnection};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Default)]
pub struct JobService {
    db: Arc<DatabaseConnection>,
}
impl JobService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        JobService { db }
    }
    pub async fn save_jobs(&self, vec_jobs: &Vec<Job>) -> Result<i32, anyhow::Error> {
        let records = vec_jobs
            .iter()
            .map(|job| jobs::ActiveModel::from(job))
            .collect::<Vec<jobs::ActiveModel>>();
        let length = records.len();
        debug!("save_jobs records:{:?}", records);

        match jobs::Entity::insert_many(records)
            .exec(self.db.as_ref())
            .await
        {
            Ok(res) => {
                log::debug!("Insert many records {:?}", length);
                Ok(res.last_insert_id)
            }
            Err(err) => {
                log::debug!("Error save_jobs {:?}", &err);
                Err(anyhow!("{:?}", &err))
            }
        }
    }
    pub async fn get_job_by_plan_ids(
        &self,
        plan_ids: &Vec<String>,
    ) -> Result<Vec<Job>, anyhow::Error> {
        let mut id_conditions = Condition::any();
        for id in plan_ids {
            id_conditions = id_conditions.add(jobs::Column::PlanId.eq(id.to_string()));
        }
        match jobs::Entity::find()
            .filter(id_conditions)
            .all(self.db.as_ref())
            .await
        {
            Ok(entities) => Ok(entities
                .iter()
                .map(|entity| Job::from((entity)))
                .collect::<Vec<Job>>()),
            Err(err) => Err(anyhow::Error::msg(format!("get_plans error: {:?}", err))),
        }
    }
}
