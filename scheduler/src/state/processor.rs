use crate::models::job_result::StoredJobResult;
use crate::models::job_result_cache::JobResultCache;
use crate::persistence::services::{JobResultService, JobService, PlanService};
use crate::report_processors::adapters::get_report_adapters;
use crate::report_processors::adapters::result_cache_appender::ResultCacheAppender;
use crate::report_processors::regular_processor::RegularReportProcessor;
use crate::report_processors::verification_processor::VerificationReportProcessor;
use crate::report_processors::ReportProcessor;
use crate::service::judgment::MainJudgment;
use common::job_manage::{JobResultDetail, JobRole};
use common::jobs::JobResult;
use diesel::PgArrayExpressionMethods;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Default)]
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
        result_service: Arc<JobResultService>,
        plan_service: Arc<PlanService>,
        job_service: Arc<JobService>,
    ) -> ProcessorState {
        //For verification processor
        let mut report_adapters = get_report_adapters(connection.clone());
        report_adapters.push(Arc::new(ResultCacheAppender::new(result_cache)));
        let judgment = MainJudgment::new(result_service.clone());
        let mut verification_processor = VerificationReportProcessor::new(
            report_adapters.clone(),
            plan_service.clone(),
            job_service.clone(),
            result_service.clone(),
            judgment.clone(),
        );
        //For regular processor
        let mut regular_processor =
            RegularReportProcessor::new(report_adapters.clone(), judgment.clone());
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
impl ProcessorState {
    pub async fn process_results(
        &mut self,
        job_results: &Vec<JobResult>,
    ) -> Result<HashMap<String, StoredJobResult>, anyhow::Error> {
        let mut map_processor_reports = HashMap::<usize, Vec<JobResultDetail>>::new();
        //Group result by processor then process result by list
        for report in job_results.iter() {
            /// add process for Regular job
            for (ind, processor) in self.processors.iter().enumerate() {
                if processor.can_apply(&report.result_detail) {
                    if let Some(mut list) = map_processor_reports.get_mut(&ind) {
                        list.push(report.result_detail.clone())
                    } else {
                        map_processor_reports
                            .insert(ind.clone(), vec![report.result_detail.clone()]);
                    }
                }
            }
        }
        if regular_results.len() > 0 {
            self.process_regular_results(regular_results);
        }
        if verification_result.len() > 0 {
            self.process_verification_results(verification_result);
        }
        Ok(HashMap::new())
    }
    pub async fn process_regular_results(
        &mut self,
        job_results: Vec<JobResult>,
    ) -> Result<HashMap<String, StoredJobResult>, anyhow::Error> {
        let mut stored_results = HashMap::<String, StoredJobResult>::new();
        let connection = self.connection.clone();
        self.regular_processor
            .process_jobs(job_results, connection)
            .await;
        Ok(HashMap::new())
    }
    pub async fn process_verification_results(
        &mut self,
        job_results: Vec<JobResult>,
    ) -> Result<HashMap<String, StoredJobResult>, anyhow::Error> {
        let mut stored_results = HashMap::<String, StoredJobResult>::new();
        for (ind, jobs) in map_processor_reports {
            let connection = self.connection.clone();
            if let Some(processor) = self.processors.get(ind) {
                processor.process_jobs(jobs, connection).await;
            };
        }
        Ok(stored_results)
    }
}
