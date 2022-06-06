use crate::report_processors::adapters::{get_report_adapters, Appender};
use crate::report_processors::ReportProcessor;
use common::job_manage::JobResult;
use sea_orm::DatabaseConnection;
pub use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct PingReportProcessor {
    report_adapters: Vec<Arc<dyn Appender>>,
}

impl PingReportProcessor {
    pub fn new() -> Self {
        let report_adapters = get_report_adapters();
        PingReportProcessor { report_adapters }
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
