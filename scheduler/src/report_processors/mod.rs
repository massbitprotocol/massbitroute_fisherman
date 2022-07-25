pub mod adapters;
pub mod channel;
pub mod dot;
pub mod eth;
pub mod regular_processor;
pub mod verification_processor;
use crate::models::job_result::StoredJobResult;
use async_trait::async_trait;
pub use channel::ReportChannel;
use common::jobs::JobResult;
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
    ) -> Result<(), anyhow::Error>;
}

// pub fn get_verification_processor(
//     connection: Arc<DatabaseConnection>,
//     result_cache: Arc<Mutex<JobResultCache>>,
// ) -> Arc<dyn ReportProcessor> {
//     let report_adapters = get_report_adapters(connection);
//     let mut report_processor = VerificationReportProcessor::new(report_adapters.clone());
//     let result_cache_adapter = Arc::new(ResultCacheAppender::new(result_cache));
//     report_processor.add_adapter(result_cache_adapter);
//     Arc::new(report_processor)
// }

// pub fn get_regular_processor(
//     connection: Arc<DatabaseConnection>,
//     result_cache: Arc<Mutex<JobResultCache>>,
// ) -> Arc<dyn ReportProcessor> {
//     let report_adapters = get_report_adapters(connection);
//     let mut report_processor = RegularReportProcessor::new(report_adapters.clone());
//     let result_cache_adapter = Arc::new(ResultCacheAppender::new(result_cache));
//     report_processor.add_adapter(result_cache_adapter);
//     Arc::new(report_processor)
// }
