pub mod adapters;
pub mod channel;
pub mod dot;
pub mod eth;
pub mod generic_processor;
use crate::models::job_result::StoredJobResult;
use crate::models::job_result_cache::JobResultCache;
use crate::report_processors::adapters::get_report_adapters;
use crate::report_processors::adapters::result_cache_appender::ResultCacheAppender;
use crate::report_processors::benchamark::BenchmarkReportProcessor;
use crate::report_processors::generic_processor::GenericReportProcessor;
use async_trait::async_trait;
pub use channel::ReportChannel;
use common::job_manage::JobResultDetail;
pub use dot::*;
pub use eth::*;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub trait ReportProcessor: Sync + Send {
    fn can_apply(&self, report: &JobResultDetail) -> bool;
    async fn process_job(
        &self,
        report: &JobResultDetail,
        db_connection: Arc<DatabaseConnection>,
    ) -> Result<StoredJobResult, anyhow::Error>;
    async fn process_jobs(
        &self,
        report: Vec<JobResultDetail>,
        db_connection: Arc<DatabaseConnection>,
    ) -> Result<Vec<StoredJobResult>, anyhow::Error>;
}

pub fn get_report_processors(
    connection: Arc<DatabaseConnection>,
    result_cache: Arc<Mutex<JobResultCache>>,
) -> Vec<Arc<dyn ReportProcessor>> {
    let report_adapters = get_report_adapters(connection);
    let mut result: Vec<Arc<dyn ReportProcessor>> = Default::default();
    let mut report_processor = GenericReportProcessor::new(report_adapters.clone());
    let result_cache_adapter = Arc::new(ResultCacheAppender::new(result_cache));
    report_processor.add_adapter(result_cache_adapter);
    result.push(Arc::new(report_processor));
    //result.push(Arc::new(PingReportProcessor::new(report_adapters.clone())));
    //result.push(Arc::new(BenchmarkReportProcessor::new()));
    //result.push(Arc::new(LatestBlockReportProcessor::new()));
    //result.push(Arc::new(RandomBlockReportProcessor::new()));
    result
}
