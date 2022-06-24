use crate::models::job_result::{ProviderTask, StoredJobResult};
use crate::report_processors::adapters::{get_report_adapters, Appender};
use crate::report_processors::ReportProcessor;
use crate::service::judgment::{JudgmentsResult, MainJudgment};
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobBenchmarkResult, JobResultDetail};
use common::jobs::JobResult;
use common::tasks::eth::JobLatestBlockResult;
use common::tasks::http_request::{JobHttpRequest, JobHttpResult};
use log::error;
use sea_orm::DatabaseConnection;
pub use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
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
    fn can_apply(&self, report: &JobResult) -> bool {
        true
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
        log::debug!("Regular report process jobs");
        let mut stored_results = Vec::<StoredJobResult>::new();
        let mut provider_task_results = HashMap::<ProviderTask, Vec<JobResult>>::new();
        for report in reports {
            let key = ProviderTask::new(
                report.provider_id.clone(),
                report.result_detail.get_name(),
                report.job_name.clone(),
            );
            let mut jobs = provider_task_results.entry(key).or_insert(Vec::default());
            jobs.push(report);
        }
        for (key, results) in provider_task_results {
            log::debug!("Process results {:?} for task {:?}", &results, &key);
            for adapter in self.report_adapters.iter() {
                adapter.append_job_results(&key, &results).await;
            }
            match self.judgment.apply_for_provider(&key, &results).await {
                Ok(res) => {}
                Err(err) => {
                    error!("{:?}", &err);
                }
            }
        }

        Ok(stored_results)
    }
}
