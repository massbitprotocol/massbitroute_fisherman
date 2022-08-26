use crate::models::job_result::ProviderTask;
use crate::models::workers::WorkerInfoStorage;
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
    worker_pool: Arc<WorkerInfoStorage>,
}

impl RegularReportProcessor {
    pub fn new(
        report_adapters: Vec<Arc<dyn Appender>>,
        judgment: MainJudgment,
        worker_pool: Arc<WorkerInfoStorage>,
    ) -> Self {
        RegularReportProcessor {
            report_adapters,
            judgment,
            worker_pool,
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

    async fn process_jobs(
        &self,
        reports: Vec<JobResult>,
        _db_connection: Arc<DatabaseConnection>,
    ) -> Result<(), anyhow::Error> {
        log::info!("Regular report process jobs");

        let mut provider_task_results = HashMap::<ProviderTask, Vec<JobResult>>::new();

        for adapter in self.report_adapters.iter() {
            log::info!(
                "With Adapter {} append {} results",
                adapter.get_name(),
                reports.len()
            );
            let res = adapter.append_job_results(&reports).await;
            if res.is_err() {
                error!("Regular append_job_results error: {res:?}");
            }
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
            match self
                .judgment
                .apply_for_regular(&key, &results, self.worker_pool.clone())
                .await
            {
                Ok(_res) => {}
                Err(err) => {
                    error!("{:?}", &err);
                }
            }
        }

        Ok(())
    }
}
