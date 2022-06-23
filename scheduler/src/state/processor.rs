use crate::models::job_result::StoredJobResult;
use crate::models::job_result_cache::JobResultCache;
use crate::persistence::services::JobResultService;
use crate::report_processors::generic_processor::VerificationReportProcessor;
use crate::report_processors::regular_processor::RegularReportProcessor;
use crate::report_processors::{
    get_regular_processor, get_verification_processor, ReportProcessor,
};
use crate::service::judgment::MainJudgment;
use common::job_manage::{JobResultDetail, JobRole};
use common::jobs::JobResult;
use diesel::PgArrayExpressionMethods;
use sea_orm::DatabaseConnection;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct ProcessorState {
    connection: Arc<DatabaseConnection>,
    regular_processor: Arc<dyn ReportProcessor>,
    verification_processor: Arc<dyn ReportProcessor>,
    result_service: Arc<JobResultService>,
    judgment: MainJudgment,
}

impl ProcessorState {
    pub fn new(
        connection: Arc<DatabaseConnection>,
        result_cache: Arc<Mutex<JobResultCache>>,
        result_service: Arc<JobResultService>,
    ) -> ProcessorState {
        let regular_processor = get_regular_processor(connection.clone(), result_cache.clone());
        let verification_processor =
            get_verification_processor(connection.clone(), result_cache.clone());
        let judgment = MainJudgment::new(result_service.clone());
        ProcessorState {
            connection,
            regular_processor,
            verification_processor,
            result_service,
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
            judgment: Default::default(),
        }
    }
}
impl ProcessorState {
    pub async fn process_results(
        &mut self,
        results: Vec<JobResult>,
    ) -> Result<HashMap<String, StoredJobResult>, anyhow::Error> {
        let mut regular_results = Vec::new();
        let mut verification_result = Vec::new();
        for result in results {
            match result.phase {
                JobRole::Regular => regular_results.push(result),
                JobRole::Verification => verification_result.push(result),
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
        Ok(HashMap::new())
    }
    pub async fn process_verification_results(
        &mut self,
        job_results: Vec<JobResult>,
    ) -> Result<HashMap<String, StoredJobResult>, anyhow::Error> {
        let mut stored_results = HashMap::<String, StoredJobResult>::new();
        let connection = self.connection.clone();
        self.verification_processor
            .process_jobs(job_results, connection)
            .await;
        Ok(stored_results)
    }
}
