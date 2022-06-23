use crate::persistence::services::job_result_service::JobResultService;
use crate::report_processors::adapters::Appender;
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::JobBenchmarkResult;
use common::jobs::JobResult;
use common::tasks::eth::JobLatestBlockResult;
use common::tasks::http_request::{JobHttpRequest, JobHttpResult};
use common::tasks::ping::JobPingResult;
use sea_orm::DatabaseConnection;
use serde_json::{Map, Value};
use std::sync::Arc;

pub struct PostgresAppender {
    job_result_service: JobResultService,
}

impl PostgresAppender {
    pub fn new(connection: Arc<DatabaseConnection>) -> Self {
        let job_result_service = JobResultService::new(connection);
        PostgresAppender { job_result_service }
    }
}

#[async_trait]
impl Appender for PostgresAppender {
    async fn append_ping_results(&self, results: &Vec<JobPingResult>) -> Result<(), Error> {
        log::debug!("PostgresAppender append ping results");
        self.job_result_service.save_result_pings(&results).await;
        Ok(())
    }
    async fn append_latest_block_results(
        &self,
        results: &Vec<JobLatestBlockResult>,
    ) -> Result<(), anyhow::Error> {
        log::debug!("PostgresAppender append ping results");
        self.job_result_service
            .save_result_latest_blocks(&results)
            .await;
        Ok(())
    }
    async fn append_benchmark_results(
        &self,
        results: &Vec<JobBenchmarkResult>,
    ) -> Result<(), anyhow::Error> {
        log::debug!("PostgresAppender append benchmark results");
        self.job_result_service
            .save_result_benchmarks(&results)
            .await;
        Ok(())
    }
    async fn append_http_request_results(
        &self,
        results: &Vec<JobResult>,
    ) -> Result<(), anyhow::Error> {
        log::debug!("PostgresAppender append benchmark results");
        self.job_result_service
            .save_result_http_requests(&results)
            .await;
        Ok(())
    }
}
