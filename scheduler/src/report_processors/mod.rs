pub mod adapters;
pub mod channel;
pub mod dot;
pub mod eth;
pub mod ping_report;

use crate::report_processors::benchamark::BenchmarkReportProcessor;
use crate::report_processors::ping_report::PingReportProcessor;
pub use channel::ReportChannel;
use common::job_manage::JobResult;
pub use dot::*;
pub use eth::*;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub trait ReportProcessor: Sync + Send {
    fn can_apply(&self, report: &JobResult) -> bool;
    fn process_job(&self, report: &JobResult, db_connection: Arc<DatabaseConnection>);
    fn process_jobs(&self, report: Vec<JobResult>, db_connection: Arc<DatabaseConnection>);
}

pub fn get_report_processors() -> Vec<Arc<dyn ReportProcessor>> {
    let mut result: Vec<Arc<dyn ReportProcessor>> = Default::default();
    result.push(Arc::new(PingReportProcessor::new()));
    result.push(Arc::new(BenchmarkReportProcessor::new()));
    result.push(Arc::new(LatestBlockReportProcessor::new()));
    result.push(Arc::new(RandomBlockReportProcessor::new()));
    result
}
