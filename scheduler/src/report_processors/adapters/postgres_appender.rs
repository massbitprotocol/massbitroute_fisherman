use crate::models::job_result::{ProviderTask, StoredJobResult};
use crate::persistence::services::job_result_service::JobResultService;
use crate::report_processors::adapters::Appender;
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobBenchmarkResult, JobResultDetail};
use common::jobs::JobResult;
use common::tasks::eth::JobLatestBlockResult;
use common::tasks::ping::JobPingResult;
use futures_util::StreamExt;
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
                    if result.job.job_name == "RoundTripTime" {
                        continue;
                    }
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
            self.job_result_service
                .save_result_http_requests(&http_request_results)
                .await;
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
    // async fn append_http_request_results(
    //     &self,
    //     results: &Vec<JobResult>,
    // ) -> Result<(), anyhow::Error> {
    //     log::debug!("PostgresAppender append benchmark results");
    //     // We do not write RoundTripTime result because there are too many.
    //     let filter_results: Vec<JobResult> = results
    //         .iter()
    //         .filter_map(|result| {
    //             if result.job_name == "RoundTripTime" {
    //                 None
    //             } else {
    //                 Some(result.clone())
    //             }
    //         })
    //         .collect();
    //     if filter_results.is_empty() {
    //         return Ok(());
    //     }
    //     self.job_result_service
    //         .save_result_http_requests(&filter_results)
    //         .await;
    //     Ok(())
    // }
}
