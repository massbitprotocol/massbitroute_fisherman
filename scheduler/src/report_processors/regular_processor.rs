use crate::models::job_result::StoredJobResult;
use crate::report_processors::adapters::{get_report_adapters, Appender};
use crate::report_processors::ReportProcessor;
use async_trait::async_trait;
use common::job_manage::{JobBenchmarkResult, JobResultDetail};
use common::jobs::JobResult;
use common::tasks::eth::JobLatestBlockResult;
use common::tasks::http_request::{JobHttpRequest, JobHttpResult};
use sea_orm::DatabaseConnection;
pub use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct RegularReportProcessor {
    report_adapters: Vec<Arc<dyn Appender>>,
}

impl RegularReportProcessor {
    pub fn new(report_adapters: Vec<Arc<dyn Appender>>) -> Self {
        RegularReportProcessor { report_adapters }
    }
    pub fn add_adapter(&mut self, adapter: Arc<dyn Appender>) {
        self.report_adapters.push(adapter);
    }
}
#[async_trait]
impl ReportProcessor for RegularReportProcessor {
    fn can_apply(&self, report: &JobResult) -> bool {
        true
    }

    async fn process_job(
        &self,
        report: &JobResult,
        db_connection: Arc<DatabaseConnection>,
    ) -> Result<StoredJobResult, anyhow::Error> {
        todo!()
    }

    async fn process_jobs(
        &self,
        reports: Vec<JobResult>,
        db_connection: Arc<DatabaseConnection>,
    ) -> Result<Vec<StoredJobResult>, anyhow::Error> {
        log::debug!("Generic report process jobs");
        let mut ping_results = Vec::new();
        let mut benchmark_results: Vec<JobBenchmarkResult> = Vec::new();
        let mut latest_block_results: Vec<JobLatestBlockResult> = Vec::new();
        let mut stored_results = Vec::<StoredJobResult>::new();
        let mut http_request_results: Vec<JobResult> = Vec::new();
        for report in reports {
            match report.result_detail {
                JobResultDetail::Ping(result) => {
                    ping_results.push(result);
                    //println!("{:?}", &ping_result);
                }
                JobResultDetail::LatestBlock(result) => latest_block_results.push(result),
                JobResultDetail::Benchmark(result) => benchmark_results.push(result),
                JobResultDetail::HttpRequest(_) => http_request_results.push(report),
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
            if http_request_results.len() > 0 {
                adapter.append_http_results(&http_request_results).await;
            }
        }
        Ok(stored_results)
    }
}
