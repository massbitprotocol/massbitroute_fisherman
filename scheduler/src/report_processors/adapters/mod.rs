use crate::models::job_result::ProviderTask;
use crate::report_processors::adapters::csv_appender::CsvAppender;
use crate::report_processors::adapters::postgres_appender::PostgresAppender;
use crate::report_processors::adapters::providers_map_appender::ProvidersMapAdapter;
use async_trait::async_trait;
use common::job_manage::JobBenchmarkResult;
use common::jobs::JobResult;
use common::tasks::eth::JobLatestBlockResult;
use common::tasks::ping::JobPingResult;
use sea_orm::DatabaseConnection;
use serde_json::{Map, Value};
use std::sync::Arc;

pub mod csv_appender;
pub mod postgres_appender;
pub mod providers_map_appender;
pub mod result_cache_appender;

#[async_trait]
pub trait Appender: Sync + Send {
    async fn append(
        &self,
        channel: String,
        report: &Map<String, Value>,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
    async fn append_job_results(
        &self,
        key: &ProviderTask,
        results: &Vec<JobResult>,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn append_ping_results(&self, results: &Vec<JobPingResult>) -> Result<(), anyhow::Error> {
        Ok(())
    }
    async fn append_latest_block_results(
        &self,
        result: &Vec<JobLatestBlockResult>,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
    async fn append_benchmark_results(
        &self,
        result: &Vec<JobBenchmarkResult>,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
    async fn append_http_request_results(
        &self,
        results: &Vec<JobResult>,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

pub fn get_report_adapters(connection: Arc<DatabaseConnection>) -> Vec<Arc<dyn Appender>> {
    let mut result: Vec<Arc<dyn Appender>> = Default::default();
    result.push(Arc::new(CsvAppender::new()));
    result.push(Arc::new(PostgresAppender::new(connection.clone())));
    result.push(Arc::new(ProvidersMapAdapter::new(connection.clone())));
    result
}
