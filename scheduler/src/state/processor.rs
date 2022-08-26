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

use crate::models::workers::WorkerInfoStorage;
use std::sync::Arc;

#[derive()]
pub struct ProcessorState {
    connection: Arc<DatabaseConnection>,
    regular_processor: Arc<dyn ReportProcessor>,
    verification_processor: Arc<dyn ReportProcessor>,
    _result_service: Arc<JobResultService>,
    _plan_service: Arc<PlanService>,
    _job_service: Arc<JobService>,
}

impl ProcessorState {
    pub fn new(
        connection: Arc<DatabaseConnection>,
        result_cache: Arc<JobResultCache>,
        plan_service: Arc<PlanService>,
        job_service: Arc<JobService>,
        result_service: Arc<JobResultService>,
        worker_pool: Arc<WorkerInfoStorage>,
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
            worker_pool.clone(),
        );
        //For regular processor
        let judgment = MainJudgment::new(result_service.clone(), &JobRole::Regular);
        let regular_processor =
            RegularReportProcessor::new(report_adapters.clone(), judgment, worker_pool);
        ProcessorState {
            connection,
            regular_processor: Arc::new(regular_processor),
            verification_processor: Arc::new(verification_processor),
            _result_service: result_service,
            _plan_service: plan_service,
            _job_service: job_service,
        }
    }
}
impl Default for ProcessorState {
    fn default() -> Self {
        ProcessorState {
            connection: Arc::new(Default::default()),
            regular_processor: Arc::new(RegularReportProcessor::default()),
            verification_processor: Arc::new(VerificationReportProcessor::default()),
            _result_service: Arc::new(Default::default()),
            _plan_service: Arc::new(Default::default()),
            _job_service: Arc::new(Default::default()),
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
        if !regular_results.is_empty() {
            let _res = self.process_regular_results(regular_results).await;
        }
        //Result of single plan
        if !verification_result.is_empty() {
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
    ) -> Result<(), anyhow::Error> {
        let connection = self.connection.clone();
        let _res = self
            .verification_processor
            .process_jobs(job_results, connection)
            .await;
        Ok(())
    }
}
