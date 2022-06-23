pub mod adapters;
pub mod channel;
pub mod dot;
pub mod eth;
pub mod generic_processor;
pub mod regular_processor;
use crate::models::job_result::StoredJobResult;
use crate::models::job_result_cache::JobResultCache;
use crate::report_processors::adapters::get_report_adapters;
use crate::report_processors::adapters::result_cache_appender::ResultCacheAppender;
use crate::report_processors::generic_processor::VerificationReportProcessor;
use crate::report_processors::regular_processor::RegularReportProcessor;
use async_trait::async_trait;
pub use channel::ReportChannel;
use common::job_manage::JobResultDetail;
use common::jobs::JobResult;
pub use dot::*;
pub use eth::*;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

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

pub fn get_verification_processor(
    connection: Arc<DatabaseConnection>,
    result_cache: Arc<Mutex<JobResultCache>>,
) -> Arc<dyn ReportProcessor> {
    let report_adapters = get_report_adapters(connection);
    let mut report_processor = VerificationReportProcessor::new(report_adapters.clone());
    let result_cache_adapter = Arc::new(ResultCacheAppender::new(result_cache));
    report_processor.add_adapter(result_cache_adapter);
    Arc::new(report_processor)
}

pub fn get_regular_processor(
    connection: Arc<DatabaseConnection>,
    result_cache: Arc<Mutex<JobResultCache>>,
) -> Arc<dyn ReportProcessor> {
    let report_adapters = get_report_adapters(connection);
    let mut report_processor = RegularReportProcessor::new(report_adapters.clone());
    let result_cache_adapter = Arc::new(ResultCacheAppender::new(result_cache));
    report_processor.add_adapter(result_cache_adapter);
    Arc::new(report_processor)
}
