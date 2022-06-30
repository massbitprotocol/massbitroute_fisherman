pub mod benchmark_judg;
pub mod http_latestblock_judg;
pub mod http_ping_judg;
pub mod latestblock_judg;
pub mod main_judg;
pub mod ping_judg;

use crate::persistence::services::job_result_service::JobResultService;
use async_trait::async_trait;
pub use benchmark_judg::BenchmarkJudgment;
use common::models::PlanEntity;
pub use latestblock_judg::LatestBlockJudgment;
pub use main_judg::MainJudgment;
pub use ping_judg::PingJudgment;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::models::job_result::ProviderTask;
use crate::service::judgment::http_latestblock_judg::HttpLatestBlockJudgment;
use crate::service::judgment::http_ping_judg::HttpPingJudgment;
use common::jobs::{Job, JobResult};
use sea_orm::DatabaseConnection;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum JudgmentsResult {
    Pass = 1,
    Failed = -1,
    Unfinished = 0,
    Error = -2,
}

impl JudgmentsResult {
    pub fn is_pass(&self) -> bool {
        self == &JudgmentsResult::Pass
    }
    pub fn is_failed(&self) -> bool {
        self == &JudgmentsResult::Failed
    }
}

#[async_trait]
pub trait ReportCheck: Sync + Send + Debug {
    fn can_apply(&self, job: &Job) -> bool;
    fn can_apply_for_result(&self, task: &ProviderTask) -> bool {
        false
    }
    async fn get_latest_judgment(
        &self,
        provider_task: &ProviderTask,
        plan: &PlanEntity,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        //Todo: implement this function
        Ok(JudgmentsResult::Pass)
    }
    /// For Verification phase
    async fn apply(
        &self,
        plan: &PlanEntity,
        job: &Vec<Job>,
    ) -> Result<JudgmentsResult, anyhow::Error>;
    /// For Regular phase
    async fn apply_for_results(
        &self,
        provider_task: &ProviderTask,
        result: &Vec<JobResult>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        Ok(JudgmentsResult::Error)
    }
}

pub fn get_report_judgments(
    config_dir: &str,
    result_service: Arc<JobResultService>,
) -> Vec<Arc<dyn ReportCheck>> {
    let mut result: Vec<Arc<dyn ReportCheck>> = Default::default();
    result.push(Arc::new(PingJudgment::new(
        config_dir,
        result_service.clone(),
    )));
    result.push(Arc::new(LatestBlockJudgment::new(
        config_dir,
        result_service.clone(),
    )));
    result.push(Arc::new(BenchmarkJudgment::new(
        config_dir,
        result_service.clone(),
    )));
    result.push(Arc::new(HttpPingJudgment::new(
        config_dir,
        result_service.clone(),
    )));
    result.push(Arc::new(HttpLatestBlockJudgment::new(
        config_dir,
        result_service.clone(),
    )));
    result
}
