use crate::persistence::JobAssignmentActiveModel;
use anyhow::anyhow;
use common::jobs::{Job, JobAssignment};
use common::{ComponentId, JobId};
use entity::seaorm::{job_assignments, jobs};
use log::debug;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::{Condition, DatabaseConnection};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Default)]
pub struct JobService {
    db: Arc<DatabaseConnection>,
}
impl JobService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        JobService { db }
    }
    pub async fn save_jobs(&self, vec_jobs: &[Job]) -> Result<i32, anyhow::Error> {
        let records = vec_jobs
            .iter()
            .map(jobs::ActiveModel::from)
            .collect::<Vec<jobs::ActiveModel>>();
        let length = records.len();
        debug!("save_jobs records:{:?}", length);

        match jobs::Entity::insert_many(records)
            .exec(self.db.as_ref())
            .await
        {
            Ok(res) => {
                log::debug!("Insert {} records", length);
                Ok(res.last_insert_id)
            }
            Err(err) => {
                log::debug!("Error save_jobs {:?}", &err);
                Err(anyhow!("{:?}", &err))
            }
        }
    }
    pub async fn save_job_assignments(
        &self,
        assignments: &[JobAssignment],
    ) -> Result<i32, anyhow::Error> {
        let models = assignments
            .iter()
            .map(JobAssignmentActiveModel::from)
            .collect::<Vec<JobAssignmentActiveModel>>();
        let length = models.len();
        debug!("save_job_assignments records:{:?}", length);
        if length > 0 {
            match job_assignments::Entity::insert_many(models)
                .exec(self.db.as_ref())
                .await
            {
                Ok(res) => {
                    log::debug!("Insert {} records", length);
                    return Ok(res.last_insert_id);
                }
                Err(err) => {
                    log::debug!("Error save_job_assignments {:?}", &err);
                    return Err(anyhow!("{:?}", &err));
                }
            }
        }
        Ok(0)
    }

    pub async fn get_job_assignments(
        &self,
    ) -> Result<HashMap<ComponentId, JobAssignment>, anyhow::Error> {
        Ok(Default::default())
    }

    pub async fn get_job_by_ids(
        &self,
        job_ids: &HashSet<JobId>,
    ) -> Result<Vec<Job>, anyhow::Error> {
        let mut id_conditions = Condition::any();
        for id in job_ids {
            id_conditions = id_conditions.add(jobs::Column::JobId.eq(id.to_string()));
        }
        match jobs::Entity::find()
            .filter(id_conditions)
            .all(self.db.as_ref())
            .await
        {
            Ok(entities) => Ok(entities.iter().map(Job::from).collect::<Vec<Job>>()),
            Err(err) => Err(anyhow::Error::msg(format!("get_plans error: {:?}", err))),
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
            Ok(entities) => Ok(entities.iter().map(Job::from).collect::<Vec<Job>>()),
            Err(err) => Err(anyhow::Error::msg(format!("get_plans error: {:?}", err))),
        }
    }
}
