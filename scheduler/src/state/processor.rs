use crate::models::job_result::StoredJobResult;
use crate::models::job_result_cache::JobResultCache;
use crate::persistence::services::{JobResultService, JobService, PlanService};
use crate::report_processors::adapters::get_report_adapters;
use crate::report_processors::adapters::result_cache_appender::ResultCacheAppender;
use crate::report_processors::regular_processor::RegularReportProcessor;
use crate::report_processors::verification_processor::VerificationReportProcessor;
use crate::report_processors::ReportProcessor;
use crate::service::judgment::MainJudgment;
use common::job_manage::JobRole;
use common::jobs::JobResult;

use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive()]
pub struct ProcessorState {
    connection: Arc<DatabaseConnection>,
    regular_processor: Arc<dyn ReportProcessor>,
    verification_processor: Arc<dyn ReportProcessor>,
    result_service: Arc<JobResultService>,
    plan_service: Arc<PlanService>,
    job_service: Arc<JobService>,
    judgment: MainJudgment,
}

impl ProcessorState {
    pub fn new(
        connection: Arc<DatabaseConnection>,
        result_cache: Arc<Mutex<JobResultCache>>,
        plan_service: Arc<PlanService>,
        job_service: Arc<JobService>,
        result_service: Arc<JobResultService>,
    ) -> ProcessorState {
        //For verification processor
        let mut report_adapters = get_report_adapters(connection.clone());
        report_adapters.push(Arc::new(ResultCacheAppender::new(result_cache.clone())));
        let verification_processor = VerificationReportProcessor::new(
            report_adapters.clone(),
            plan_service.clone(),
            job_service.clone(),
            result_service.clone(),
            result_cache,
            MainJudgment::new(result_service.clone(), &JobRole::Verification),
        );
        //For regular processor
        let judgment = MainJudgment::new(result_service.clone(), &JobRole::Regular);
        let regular_processor = RegularReportProcessor::new(report_adapters.clone(), judgment);
        let judgment = MainJudgment::new(result_service.clone(), &JobRole::Regular);
        ProcessorState {
            connection,
            regular_processor: Arc::new(regular_processor),
            verification_processor: Arc::new(verification_processor),
            result_service,
            plan_service,
            job_service,
            judgment,
        }
    }
}
impl Default for ProcessorState {
    fn default() -> Self {
        ProcessorState {
            connection: Arc::new(Default::default()),
            regular_processor: Arc::new(RegularReportProcessor::default()),
            verification_processor: Arc::new(VerificationReportProcessor::default()),
            result_service: Arc::new(Default::default()),
            plan_service: Arc::new(Default::default()),
            job_service: Arc::new(Default::default()),
            judgment: Default::default(),
        }
    }
}
impl ProcessorState {}

impl ProcessorState {
    pub async fn process_results(&self, results: Vec<JobResult>) -> Result<(), anyhow::Error> {
        let mut regular_results = Vec::new();
        let mut verification_result = Vec::new();
        for result in results {
            match result.phase {
                JobRole::Regular => regular_results.push(result),
                JobRole::Verification => verification_result.push(result),
            }
        }
        if regular_results.len() > 0 {
            let _res = self.process_regular_results(regular_results).await;
        }
        //Result of single plan
        if verification_result.len() > 0 {
            let _res = self.process_verification_results(verification_result).await;
        }
        Ok(())
    }
    pub async fn process_regular_results(
        &self,
        job_results: Vec<JobResult>,
    ) -> Result<(), anyhow::Error> {
        let connection = self.connection.clone();
        self.regular_processor
            .process_jobs(job_results, connection)
            .await?;
        Ok(())
    }
    pub async fn process_verification_results(
        &self,
        job_results: Vec<JobResult>,
    ) -> Result<HashMap<String, StoredJobResult>, anyhow::Error> {
        let stored_results = HashMap::<String, StoredJobResult>::new();
        let connection = self.connection.clone();
        let _res = self
            .verification_processor
            .process_jobs(job_results, connection)
            .await;
        Ok(stored_results)
    }
}
