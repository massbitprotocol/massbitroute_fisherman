use crate::persistence::seaorm::job_result_pings::Model as ResultPingModel;
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
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use sea_orm::{Condition, DatabaseConnection};
use std::collections::{HashMap, HashSet};
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
    ) -> Result<usize, anyhow::Error> {
        let job_ids = Vec::from_iter(
            vec_results
                .iter()
                .map(|res| res.job.job_id.clone())
                .collect::<HashSet<String>>(),
        );
        let res_results = self
            .get_result_ping_by_job_ids(&job_ids)
            .await
            .unwrap_or(Vec::new());
        let mut map_results = HashMap::<String, ResultPingModel>::new();
        for res in res_results.into_iter() {
            map_results.insert(res.job_id.clone(), res);
        }
        let mut map_update_results = HashMap::<String, bool>::new();
        /*
        let mut map_job_ids = HashMap::<String, Vec<String>>::default();
        for job_res in vec_results.iter() {
            if let Some(job_ids) = map_job_ids.get_mut(&job_res.worker_id) {
                job_ids.push(job_res.job.job_id.clone());
            } else {
                map_job_ids.insert(job_res.worker_id.clone(), vec![job_res.job.job_id.clone()]);
            }
        }
        */
        let mut new_records = HashMap::<String, ResultPingModel>::default();
        for result in vec_results.iter() {
            if let Some(res) = map_results.get_mut(&result.job.job_id) {
                res.add_response_time(result.response.response_time as i64);
                map_update_results.insert(result.job.job_id.clone(), true);
            } else {
                new_records.insert(result.job.job_id.clone(), ResultPingModel::from(result));
            }
        }

        let len = new_records.len();
        //New records
        let active_models = new_records
            .into_iter()
            .map(|(key, val)| job_result_pings::ActiveModel::from_model(&val))
            .collect::<Vec<job_result_pings::ActiveModel>>();
        debug!(
            "Create new {:?} records for job_result_pings",
            &active_models
        );
        let tnx = self.db.begin().await?;
        for (id, _) in map_update_results.iter() {
            let model = map_results.remove(id).unwrap();
            let response_times = model.response_times.clone();
            let mut active_model: job_result_pings::ActiveModel = model.into();
            active_model.response_times = Set(response_times);
            log::debug!("Update model {:?}", &active_model);
            match active_model.update(self.db.as_ref()).await {
                Ok(_) => {
                    log::debug!("Update successfully");
                }
                Err(err) => {
                    log::error!("{:?}", &err);
                }
            }
        }
        if active_models.len() > 0 {
            log::debug!(
                "Insert job_ping_result {:?} records: {:?}",
                len,
                &active_models
            );
            match job_result_pings::Entity::insert_many(active_models)
                .exec(self.db.as_ref())
                .await
            {
                Ok(res) => {
                    log::debug!("Insert many records {:?}", len);
                }
                Err(err) => {
                    log::debug!("Error {:?}", &err);
                }
            }
        }
        match tnx.commit().await {
            Ok(_) => {
                log::debug!("Transaction commited successful");
                Ok(len + map_update_results.len())
            }
            Err(err) => {
                log::debug!("Transaction commited with error {:?}", &err);
                Err(anyhow!("{:?}", &err))
            }
        }
    }
    pub async fn get_result_ping_by_job_ids(
        &self,
        job_ids: &Vec<String>,
    ) -> Result<Vec<ResultPingModel>, anyhow::Error> {
        let mut id_conditions = Condition::any();
        for id in job_ids {
            id_conditions = id_conditions.add(job_result_pings::Column::JobId.eq(id.to_string()));
        }
        job_result_pings::Entity::find()
            .filter(id_conditions)
            .all(self.db.as_ref())
            .await
            .map_err(|err| anyhow!("{:?}", &err))
        /*
        {
            Ok(results) => {
                let res = results
                    .iter()
                    .map(|model| JobPingResult.from(model))
                    .collect::<Vec<JobPingResult>>();
                Ok(res)
            }
            Err(err) => Err(anyhow!("{:?}", &err)),
        }
        */
    }
    /*
     * map by worker_id and vector of response times
     */
    pub async fn get_result_pings(&self, job_id: &str) -> Result<Vec<i64>, anyhow::Error> {
        match job_result_pings::Entity::find()
            .filter(job_result_pings::Column::JobId.eq(job_id.to_owned()))
            .all(self.db.as_ref())
            .await
        {
            Ok(results) => results
                .get(0)
                .and_then(|model| {
                    let response_times: serde_json::Value = model.response_times.clone();
                    let values = response_times
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|val| val.as_i64().unwrap())
                        .collect::<Vec<i64>>();
                    Some(values)
                })
                .ok_or(anyhow!("Result not found for job {:?}", job_id)),
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
                    //res.push(JobBenchmarkResult::from(model))
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
                    //res.push(JobLatestBlockResult::from(model))
                }
                Ok(res)
            }
            Err(err) => Err(anyhow!("{:?}", &err)),
        }
    }
}
