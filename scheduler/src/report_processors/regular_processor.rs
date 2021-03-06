use crate::models::job_result::{ProviderTask, StoredJobResult};
use crate::report_processors::adapters::Appender;
use crate::report_processors::ReportProcessor;
use crate::service::judgment::MainJudgment;
use async_trait::async_trait;
use common::jobs::JobResult;
use log::error;
use sea_orm::DatabaseConnection;
pub use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct RegularReportProcessor {
    report_adapters: Vec<Arc<dyn Appender>>,
    judgment: MainJudgment,
}

impl RegularReportProcessor {
    pub fn new(report_adapters: Vec<Arc<dyn Appender>>, judgment: MainJudgment) -> Self {
        RegularReportProcessor {
            report_adapters,
            judgment,
        }
    }
    pub fn add_adapter(&mut self, adapter: Arc<dyn Appender>) {
        self.report_adapters.push(adapter);
    }
}
#[async_trait]
impl ReportProcessor for RegularReportProcessor {
    fn can_apply(&self, _report: &JobResult) -> bool {
        true
    }

    async fn process_job(
        &self,
        _report: &JobResult,
        _db_connection: Arc<DatabaseConnection>,
    ) -> Result<StoredJobResult, anyhow::Error> {
        todo!()
    }

    async fn process_jobs(
        &self,
        reports: Vec<JobResult>,
        _db_connection: Arc<DatabaseConnection>,
    ) -> Result<(), anyhow::Error> {
        log::info!("Regular report process jobs");
        //let stored_results = Vec::<StoredJobResult>::new();
        let mut provider_task_results = HashMap::<ProviderTask, Vec<JobResult>>::new();

        for adapter in self.report_adapters.iter() {
            log::info!(
                "With Adapter {} append {} results",
                adapter.get_name(),
                reports.len()
            );
            adapter.append_job_results(&reports).await;
        }

        for report in reports {
            let key = ProviderTask::new(
                report.provider_id.clone(),
                report.provider_type.clone(),
                report.result_detail.get_name(),
                report.job_name.clone(),
            );
            let jobs = provider_task_results.entry(key).or_insert(Vec::default());
            jobs.push(report);
        }
        for (key, results) in provider_task_results {
            match self.judgment.apply_for_regular(&key, &results).await {
                Ok(_res) => {}
                Err(err) => {
                    error!("{:?}", &err);
                }
            }
        }

        Ok(())
    }
}
