use crate::report_processors::ReportProcessor;
use common::job_manage::JobResult;
use sea_orm::DatabaseConnection;
pub use serde::{Deserialize, Serialize};
use std::pin::pin;
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct PingReportProcessor {}

impl PingReportProcessor {
    pub fn new() -> Self {
        PingReportProcessor {}
    }
}
impl ReportProcessor for PingReportProcessor {
    fn can_apply(&self, report: &JobResult) -> bool {
        true
    }

    fn process_job(&self, report: &JobResult, db_connection: Arc<DatabaseConnection>) {
        todo!()
    }

    fn process_jobs(&self, reports: Vec<JobResult>, db_connection: Arc<DatabaseConnection>) {
        for report in reports {
            match report {
                JobResult::Ping(ping_result) => {}
                _ => {}
            }
        }
    }
}
