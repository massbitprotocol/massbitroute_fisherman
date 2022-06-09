use crate::report_processors::ReportProcessor;
use async_trait::async_trait;
use common::job_manage::JobResult;
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
            JobResult::Benchmark(_) => true,
            _ => false,
        }
    }

    async fn process_job(&self, report: &JobResult, db_connection: Arc<DatabaseConnection>) {
        todo!()
    }

    async fn process_jobs(&self, report: Vec<JobResult>, db_connection: Arc<DatabaseConnection>) {
        todo!()
    }
}
