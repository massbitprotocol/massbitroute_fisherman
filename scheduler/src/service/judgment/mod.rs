pub mod benchmark_judg;
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

use common::jobs::{Job, JobResult};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

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
    fn can_apply_for_result(&self, job_name: &String) -> bool {
        false
    }
    /*
     * result > 0 result is looked good
     * result = 0 job is not finished
     * result < 0 something is bad
     */
    async fn apply(&self, plan: &PlanEntity, job: &Job) -> Result<JudgmentsResult, anyhow::Error>;
    async fn apply_for_results(
        &self,
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
    result
}
