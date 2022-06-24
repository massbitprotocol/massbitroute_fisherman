use crate::models::job_result::StoredJobResult;
use crate::report_processors::ReportProcessor;
use async_trait::async_trait;
use common::job_manage::JobResultDetail;
use common::jobs::JobResult;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct BenchmarkReportProcessor {}

impl BenchmarkReportProcessor {
    pub fn new() -> Self {
        BenchmarkReportProcessor {}
    }
}
#[async_trait]
impl ReportProcessor for BenchmarkReportProcessor {
    fn can_apply(&self, report: &JobResult) -> bool {
        match report {
            JobResultDetail::Benchmark(_) => true,
            _ => false,
        }
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
        report: Vec<JobResult>,
        db_connection: Arc<DatabaseConnection>,
    ) -> Result<Vec<StoredJobResult>, anyhow::Error> {
        todo!()
    }
}
