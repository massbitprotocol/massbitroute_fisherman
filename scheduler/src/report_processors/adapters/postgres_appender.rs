use crate::models::job_result::StoredJobResult;
use crate::persistence::services::job_result_service::JobResultService;
use crate::report_processors::adapters::helper::LatestBlockCache;
use crate::report_processors::adapters::Appender;

use async_trait::async_trait;
use common::job_manage::{JobBenchmarkResult, JobResultDetail};
use common::jobs::JobResult;
use common::tasks::eth::JobLatestBlockResult;

use log::debug;

use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, Value,
};

use std::sync::Arc;
use tokio::sync::Mutex;

const ROUND_TRIP_TIME: &str = "RoundTripTime";
const LATEST_BLOCK: &str = "LatestBlock";
const INSERT_PROVIDER_LATEST_BLOCK: &str = r#"INSERT INTO provider_latest_blocks
(provider_id, blockchain, network, blockhash, block_timestamp, max_block_timestamp, 
block_number, max_block_number, response_timestamp)"#;
const CONFLICT_PROVIDER_LATEST_BLOCK: &str = r#"ON CONFLICT ON CONSTRAINT provider_latest_blocks_provider_id_uindex
                                DO UPDATE SET blockhash = EXCLUDED.blockhash
                                              ,block_timestamp= EXCLUDED.block_timestamp
                                              ,max_block_timestamp = EXCLUDED.max_block_timestamp
                                              ,block_number = EXCLUDED.block_number
                                              ,max_block_number = EXCLUDED.max_block_number
                                              ,response_timestamp = EXCLUDED.response_timestamp;"#;

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
        let _stored_results = Vec::<StoredJobResult>::new();
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
                JobResultDetail::HttpRequest(_) => {
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
            self.append_http_request_results(http_request_results).await;
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
    async fn check_flush_latest_block_cache(&self) -> Result<u64, anyhow::Error> {
        let mut cache = self.latest_block_cache.lock().await;
        let data = cache.get_cache_data_for_flushing();
        let length = data.len();
        if length > 0 {
            let mut values = Vec::new();
            let mut place_holders = Vec::new();
            let mut row = 1;
            let column_count = 9;
            for item in data {
                let mut rows = Vec::new();
                values.push(Value::from(item.provider_id.clone()));
                values.push(Value::from(item.blockchain.clone()));
                values.push(Value::from(item.network.clone()));
                values.push(Value::from(item.blockhash.clone()));
                values.push(Value::from(item.block_timestamp.clone()));
                values.push(Value::from(item.max_block_timestamp.clone()));
                values.push(Value::from(item.block_number.clone()));
                values.push(Value::from(item.max_block_number.clone()));
                values.push(Value::from(item.response_timestamp.clone()));
                for i in 0..column_count {
                    rows.push(format!("${}", row + i));
                }
                row = row + column_count;
                place_holders.push(format!("({})", rows.join(",")));
            }
            let query = format!(
                "{} VALUES {} {}",
                INSERT_PROVIDER_LATEST_BLOCK,
                place_holders.join(","),
                CONFLICT_PROVIDER_LATEST_BLOCK
            );
            debug!("Flush latest block query: {}", query.as_str());
            match self
                .connection
                .as_ref()
                .execute(Statement::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    query.as_str(),
                    values,
                ))
                .await
            {
                Ok(exec_res) => {
                    log::debug!("{:?}", &exec_res);
                    Ok(exec_res.rows_affected())
                }
                Err(err) => {
                    log::error!("Upsert error: {:?}", &err);
                    Ok(0)
                }
            }
        } else {
            debug!("No data to flush to latest blocks table");
            Ok(0)
        }
    }
}
