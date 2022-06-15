pub mod adapters;
pub mod channel;
pub mod dot;
pub mod eth;
pub mod generic_processor;
pub mod ping_report;
use crate::models::job_result::StoredJobResult;
use crate::report_processors::adapters::get_report_adapters;
use crate::report_processors::benchamark::BenchmarkReportProcessor;
use crate::report_processors::generic_processor::GenericReportProcessor;
use crate::report_processors::ping_report::PingReportProcessor;
use async_trait::async_trait;
pub use channel::ReportChannel;
use common::job_manage::JobResult;
pub use dot::*;
pub use eth::*;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[async_trait]
pub trait ReportProcessor: Sync + Send {
    fn can_apply(&self, report: &JobResult) -> bool;
    async fn process_job(
        &self,
        report: &JobResult,
        db_connection: Arc<DatabaseConnection>,
    ) -> Result<StoredJobResult, anyhow::Error>;
    async fn process_jobs(
        &self,
        report: Vec<JobResult>,
        db_connection: Arc<DatabaseConnection>,
    ) -> Result<Vec<StoredJobResult>, anyhow::Error>;
}

pub fn get_report_processors(connection: Arc<DatabaseConnection>) -> Vec<Arc<dyn ReportProcessor>> {
    let report_adapters = get_report_adapters(connection);
    let mut result: Vec<Arc<dyn ReportProcessor>> = Default::default();
    result.push(Arc::new(GenericReportProcessor::new(
        report_adapters.clone(),
    )));
    //result.push(Arc::new(PingReportProcessor::new(report_adapters.clone())));
    //result.push(Arc::new(BenchmarkReportProcessor::new()));
    //result.push(Arc::new(LatestBlockReportProcessor::new()));
    //result.push(Arc::new(RandomBlockReportProcessor::new()));
    result
}
