use crate::persistence::seaorm::jobs::Model;
use crate::persistence::seaorm::{
    job_result_benchmarks, job_result_latest_blocks, job_result_pings, jobs,
};
use anyhow::anyhow;
use common::component::Zone;
use common::job_manage::{Job, JobBenchmarkResult};
use common::tasks::eth::JobLatestBlockResult;
use common::tasks::ping::JobPingResult;
use common::worker::WorkerInfo;
use log::{debug, error, log};
use sea_orm::DatabaseConnection;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Default)]
pub struct JobResultService {
    db: Arc<DatabaseConnection>,
}
impl JobResultService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        JobResultService { db }
    }
    pub async fn save_result_pings(
        &self,
        vec_results: &Vec<JobPingResult>,
    ) -> Result<i64, anyhow::Error> {
        let records = vec_results
            .iter()
            .map(|job| job_result_pings::ActiveModel::from(job))
            .collect::<Vec<job_result_pings::ActiveModel>>();
        let length = records.len();
        debug!("Save job_result_pings with {:?} records", records);

        match job_result_pings::Entity::insert_many(records)
            .exec(self.db.as_ref())
            .await
        {
            Ok(res) => {
                log::debug!("Insert many records {:?}", length);
                Ok(res.last_insert_id)
            }
            Err(err) => {
                log::debug!("Error {:?}", &err);
                Err(anyhow!("{:?}", &err))
            }
        }
    }
    pub async fn get_result_pings(
        &self,
        plan_id: &str,
    ) -> Result<HashMap<String, Vec<i64>>, anyhow::Error> {
        match job_result_pings::Entity::find()
            .filter(job_result_pings::Column::PlanId.eq(plan_id.to_owned()))
            .all(self.db.as_ref())
            .await
        {
            Ok(results) => {
                let mut res = HashMap::<String, Vec<i64>>::new();
                for model in results.iter() {
                    let worker_id = model.worker_id.clone();
                    let response_time = model.response_time.clone();
                    if let Some(mut vec) = res.get_mut(&worker_id) {
                        vec.push(response_time);
                    } else {
                        res.push(worker_id.clone(), response_time);
                    }
                    res.push(JobPingResult::from(model))
                }
                Ok(res)
            }
            Err(err) => Err(anyhow!("{:?}", &err)),
        }
    }

    pub async fn save_result_benchmarks(
        &self,
        vec_results: &Vec<JobBenchmarkResult>,
    ) -> Result<i64, anyhow::Error> {
        let records = vec_results
            .iter()
            .map(|job| job_result_benchmarks::ActiveModel::from(job))
            .collect::<Vec<job_result_benchmarks::ActiveModel>>();
        let length = records.len();
        debug!("Save job_result_benchmarks with {:?} records", records);

        match job_result_benchmarks::Entity::insert_many(records)
            .exec(self.db.as_ref())
            .await
        {
            Ok(res) => {
                log::debug!("Insert many records {:?}", length);
                Ok(res.last_insert_id)
            }
            Err(err) => {
                log::debug!("Error {:?}", &err);
                Err(anyhow!("{:?}", &err))
            }
        }
    }
    pub async fn save_result_latest_blocks(
        &self,
        vec_results: &Vec<JobLatestBlockResult>,
    ) -> Result<i64, anyhow::Error> {
        let records = vec_results
            .iter()
            .map(|job| job_result_latest_blocks::ActiveModel::from(job))
            .collect::<Vec<job_result_latest_blocks::ActiveModel>>();
        let length = records.len();
        debug!("Save job_result_latest_blocks with {:?} records", records);

        match job_result_latest_blocks::Entity::insert_many(records)
            .exec(self.db.as_ref())
            .await
        {
            Ok(res) => {
                log::debug!("Insert many records {:?}", length);
                Ok(res.last_insert_id)
            }
            Err(err) => {
                log::debug!("Error {:?}", &err);
                Err(anyhow!("{:?}", &err))
            }
        }
    }
    pub async fn get_result_benchmarks(
        &self,
        plan_id: &str,
    ) -> Result<Vec<JobBenchmarkResult>, anyhow::Error> {
        match job_result_benchmarks::Entity::find()
            .filter(job_result_benchmarks::Column::PlanId.eq(plan_id.to_owned()))
            .all(self.db.as_ref())
            .await
        {
            Ok(results) => {
                let mut res = Vec::new();
                for model in results.iter() {
                    res.push(JobBenchmarkResult::from(model))
                }
                Ok(res)
            }
            Err(err) => Err(anyhow!("{:?}", &err)),
        }
    }
    pub async fn get_result_latest_blocks(
        &self,
        plan_id: &str,
    ) -> Result<Vec<JobLatestBlockResult>, anyhow::Error> {
        match job_result_latest_blocks::Entity::find()
            .filter(job_result_latest_blocks::Column::PlanId.eq(plan_id.to_owned()))
            .all(self.db.as_ref())
            .await
        {
            Ok(results) => {
                let mut res = Vec::new();
                for model in results.iter() {
                    res.push(JobLatestBlockResult::from(model))
                }
                Ok(res)
            }
            Err(err) => Err(anyhow!("{:?}", &err)),
        }
    }
}
