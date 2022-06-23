use crate::models::job_result::StoredJobResult;
use crate::report_processors::adapters::{get_report_adapters, Appender};
use crate::report_processors::ReportProcessor;
use async_trait::async_trait;
use common::job_manage::{JobBenchmarkResult, JobResultDetail};
use common::tasks::eth::JobLatestBlockResult;
use sea_orm::DatabaseConnection;
pub use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct GenericReportProcessor {
    report_adapters: Vec<Arc<dyn Appender>>,
}

impl GenericReportProcessor {
    pub fn new(report_adapters: Vec<Arc<dyn Appender>>) -> Self {
        GenericReportProcessor { report_adapters }
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
        log::debug!("Generic report process jobs");
        let mut ping_results = Vec::new();
        let mut benchmark_results: Vec<JobBenchmarkResult> = Vec::new();
        let mut latest_block_results: Vec<JobLatestBlockResult> = Vec::new();
        let mut stored_results = Vec::<StoredJobResult>::new();
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
        Ok(stored_results)
    }
}
