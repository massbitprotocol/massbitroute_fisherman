use crate::models::job_result::{ProviderTask, StoredJobResult};
use crate::persistence::services::job_result_service::JobResultService;
use crate::report_processors::adapters::helper::LatestBlockCache;
use crate::report_processors::adapters::Appender;
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use common::job_manage::{JobBenchmarkResult, JobResultDetail};
use common::jobs::JobResult;
use common::tasks::eth::JobLatestBlockResult;
use common::tasks::ping::JobPingResult;
use entity::seaorm::provider_latest_blocks::ActiveModel as ProviderLatestBlockActiveModel;
use entity::seaorm::provider_latest_blocks::Entity as ProviderLatestBlockEntity;
use futures_util::StreamExt;
use log::debug;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait,
};
use serde_json::{Map, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

const ROUND_TRIP_TIME: &str = "RoundTripTime";
const LATEST_BLOCK: &str = "LatestBlock";

pub struct PostgresAppender {
    connection: Arc<DatabaseConnection>,
    job_result_service: JobResultService,
    latest_block_cache: Mutex<LatestBlockCache>,
}

impl PostgresAppender {
    pub fn new(connection: Arc<DatabaseConnection>) -> Self {
        let job_result_service = JobResultService::new(connection.clone());
        PostgresAppender {
            connection,
            job_result_service,
            latest_block_cache: Mutex::new(LatestBlockCache::default()),
        }
    }
}

#[async_trait]
impl Appender for PostgresAppender {
    fn get_name(&self) -> String {
        "PostgresAppender".to_string()
    }
    async fn append_job_results(&self, reports: &Vec<JobResult>) -> Result<(), anyhow::Error> {
        let mut ping_results = Vec::new();
        let mut benchmark_results: Vec<JobBenchmarkResult> = Vec::new();
        let mut latest_block_results: Vec<JobLatestBlockResult> = Vec::new();
        let mut stored_results = Vec::<StoredJobResult>::new();
        let mut http_request_results: Vec<JobResult> = Vec::new();
        for report in reports {
            match &report.result_detail {
                JobResultDetail::Ping(result) => {
                    ping_results.push(result.clone());
                }
                JobResultDetail::LatestBlock(result) => {
                    latest_block_results.push(result.clone());
                }
                JobResultDetail::Benchmark(result) => {
                    benchmark_results.push(result.clone());
                }
                JobResultDetail::HttpRequest(ref result) => {
                    http_request_results.push(report.clone());
                }
                _ => {}
            }
        }
        //update provider map base on ping result
        // Todo: Add response time for each Job result
        if ping_results.len() > 0 {
            self.job_result_service
                .save_result_pings(&ping_results)
                .await;
        }
        if latest_block_results.len() > 0 {
            self.job_result_service
                .save_result_latest_blocks(&latest_block_results)
                .await;
        }
        if benchmark_results.len() > 0 {
            self.job_result_service
                .save_result_benchmarks(&benchmark_results)
                .await;
        }
        if http_request_results.len() > 0 {
            self.append_http_request_results(http_request_results);
        }
        Ok(())
    }
    // async fn append_ping_results(&self, results: &Vec<JobPingResult>) -> Result<(), Error> {
    //     log::debug!("PostgresAppender append ping results");
    //     self.job_result_service.save_result_pings(&results).await;
    //     Ok(())
    // }
    // async fn append_latest_block_results(
    //     &self,
    //     results: &Vec<JobLatestBlockResult>,
    // ) -> Result<(), anyhow::Error> {
    //     log::debug!("PostgresAppender append ping results");
    //     self.job_result_service
    //         .save_result_latest_blocks(&results)
    //         .await;
    //     Ok(())
    // }
    // async fn append_benchmark_results(
    //     &self,
    //     results: &Vec<JobBenchmarkResult>,
    // ) -> Result<(), anyhow::Error> {
    //     log::debug!("PostgresAppender append benchmark results");
    //     self.job_result_service
    //         .save_result_benchmarks(&results)
    //         .await;
    //     Ok(())
    // }
}

impl PostgresAppender {
    async fn append_http_request_results(
        &self,
        results: Vec<JobResult>,
    ) -> Result<(), anyhow::Error> {
        log::debug!("PostgresAppender append http request results");
        // We do not write RoundTripTime result because there are too many.
        let mut latest_blocks: Vec<JobResult> = Vec::new();
        let mut rtts: Vec<JobResult> = Vec::new();
        let mut generals: Vec<JobResult> = Vec::new();
        for result in results {
            match result.job_name.as_str() {
                ROUND_TRIP_TIME => {}
                LATEST_BLOCK => {
                    self.latest_block_cache.lock().await.append_result(result);
                }
                _ => {
                    generals.push(result);
                }
            }
        }
        self.check_flush_latest_block_cache().await;
        if generals.len() > 0 {
            self.job_result_service
                .save_result_http_requests(&generals)
                .await;
        }
        Ok(())
    }
    async fn check_flush_latest_block_cache(&self) -> Result<usize, anyhow::Error> {
        let mut cache = self.latest_block_cache.lock().await;
        let data = cache.get_cache_data_for_flushing();
        let length = data.len();
        if length > 0 {
            let records = data
                .iter()
                .map(|item| LatestBlockCache::create_active_model(item))
                .collect::<Vec<ProviderLatestBlockActiveModel>>();

            debug!("Save {:?} records to table provider_latest_blocks", length);

            match ProviderLatestBlockEntity::insert_many(records)
                .exec(self.connection.as_ref())
                .await
            {
                Ok(res) => {
                    log::debug!("Insert many records {:?}", length);
                    Ok(res.last_insert_id as usize)
                }
                Err(err) => {
                    log::debug!("Error {:?}", &err);
                    Err(anyhow!("{:?}", &err))
                }
            }
        } else {
            Ok(0)
        }
    }
}
