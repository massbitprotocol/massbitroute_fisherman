use crate::models::job_result::StoredJobResult;
use crate::report_processors::adapters::{get_report_adapters, Appender};
use crate::report_processors::ReportProcessor;
use async_trait::async_trait;
use common::job_manage::JobResult;
use sea_orm::DatabaseConnection;
pub use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct PingReportProcessor {
    report_adapters: Vec<Arc<dyn Appender>>,
}

impl PingReportProcessor {
    pub fn new(report_adapters: Vec<Arc<dyn Appender>>) -> Self {
        PingReportProcessor { report_adapters }
    }
}
#[async_trait]
impl ReportProcessor for PingReportProcessor {
    fn can_apply(&self, report: &JobResult) -> bool {
        match report {
            JobResult::Ping(_) => true,
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
        reports: Vec<JobResult>,
        db_connection: Arc<DatabaseConnection>,
    ) -> Result<Vec<StoredJobResult>, anyhow::Error> {
        let mut ping_results = Vec::new();
        for report in reports {
            match report {
                JobResult::Ping(result) => {
                    ping_results.push(result);
                }
                _ => {}
            }
        }
        for adapter in self.report_adapters.iter() {
            adapter.append_ping_results(&ping_results).await;
        }
        //Todo: use generic processor
        Ok(Vec::new())
    }
}
