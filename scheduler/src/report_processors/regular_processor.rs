use crate::models::job_result::StoredJobResult;
use crate::report_processors::adapters::{get_report_adapters, Appender};
use crate::report_processors::ReportProcessor;
use crate::service::judgment::{JudgmentsResult, MainJudgment};
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobBenchmarkResult, JobResultDetail};
use common::tasks::eth::JobLatestBlockResult;
use common::tasks::http_request::{JobHttpRequest, JobHttpResult};
use log::error;
use sea_orm::DatabaseConnection;
pub use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct GenericReportProcessor {
    report_adapters: Vec<Arc<dyn Appender>>,
    judgment: MainJudgment,
}

impl RegularReportProcessor {
    pub fn new(report_adapters: Vec<Arc<dyn Appender>>, judgment: MainJudgment) -> Self {
        RegularReportProcessor {
            report_adapters,
            judgment,
        }
    }
    pub fn add_adapter(&mut self, adapter: Arc<dyn Appender>) {
        self.report_adapters.push(adapter);
    }
}
#[async_trait]
impl ReportProcessor for GenericReportProcessor {
    fn can_apply(&self, report: &JobResultDetail) -> bool {
        true
    }

    async fn process_job(
        &self,
        report: &JobResultDetail,
        db_connection: Arc<DatabaseConnection>,
    ) -> Result<StoredJobResult, anyhow::Error> {
        todo!()
    }

    async fn process_jobs(
        &self,
        reports: Vec<JobResultDetail>,
        db_connection: Arc<DatabaseConnection>,
    ) -> Result<Vec<StoredJobResult>, anyhow::Error> {
        log::debug!("Regular report process jobs");
        let mut stored_results = Vec::<StoredJobResult>::new();
        let mut ping_results = Vec::new();
        let mut benchmark_results: Vec<JobBenchmarkResult> = Vec::new();
        let mut latest_block_results: Vec<JobLatestBlockResult> = Vec::new();
        let mut http_request_results: Vec<JobResult> = Vec::new();
        for report in reports {
            match report {
                JobResultDetail::Ping(result) => {
                    ping_results.push(result);
                    //println!("{:?}", &ping_result);
                }
                JobResultDetail::LatestBlock(result) => latest_block_results.push(result),
                JobResultDetail::Benchmark(result) => benchmark_results.push(result),
                _ => {}
            }
        }
        //update provider map base on ping result
        // Todo: Add response time for each Job result
        for adapter in self.report_adapters.iter() {
            if ping_results.len() > 0 {
                adapter.append_ping_results(&ping_results).await;
            }
            if latest_block_results.len() > 0 {
                adapter
                    .append_latest_block_results(&latest_block_results)
                    .await;
            }
            if benchmark_results.len() > 0 {
                adapter.append_benchmark_results(&benchmark_results).await;
            }
        }
        if http_request_results.len() > 0 {
            for adapter in self.report_adapters.iter() {
                adapter
                    .append_http_request_results(&http_request_results)
                    .await;
            }
            self.judg_http_request_results(http_request_results).await;
        }

        Ok(stored_results)
    }
}

impl RegularReportProcessor {
    pub async fn judg_http_request_results(&self, results: Vec<JobResult>) {
        //Separate result by provider id
        let mut map_results = HashMap::<String, Vec<JobResult>>::new();
        for result in results {
            let mut results = map_results
                .entry(result.provider_id.clone())
                .or_insert(Vec::default());
            results.push(result);
        }
        for (provider_id, results) in map_results.into_iter() {
            if results.len() > 0 {
                match self.judgment.apply_for_provider(provider_id, results).await {
                    Ok(res) => {}
                    Err(err) => {
                        error!("{:?}", &err);
                    }
                }
            }
        }
    }
}
