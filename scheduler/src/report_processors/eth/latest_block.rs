use crate::report_processors::ReportProcessor;
use common::job_manage::JobResult;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct LatestBlockReportProcessor {}

impl LatestBlockReportProcessor {
    pub fn new() -> Self {
        LatestBlockReportProcessor {}
    }
}
impl ReportProcessor for LatestBlockReportProcessor {
    fn can_apply(&self, report: &JobResult) -> bool {
        true
    }

    fn process_job(&self, report: &JobResult, db_connection: Arc<DatabaseConnection>) {
        todo!()
    }

    fn process_jobs(&self, report: Vec<JobResult>, db_connection: Arc<DatabaseConnection>) {
        todo!()
    }
}
