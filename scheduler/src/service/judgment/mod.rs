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

use common::component::ComponentType;
use common::job_manage::{Job, JobRole};
use common::{Deserialize, Serialize};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum JudgmentsResult {
    Pass = 1,
    Failed = -1,
    Unfinished = 0,
}

#[async_trait]
pub trait ReportCheck: Sync + Send {
    fn can_apply(&self, job: &Job) -> bool;
    /*
     * result > 0 result is looked good
     * result = 0 job is not finished
     * result < 0 something is bad
     */
    async fn apply(&self, plan: &PlanEntity, job: &Job) -> Result<JudgmentsResult, anyhow::Error>;
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
    result.push(Arc::new(LatestBlockJudgment::new(result_service.clone())));
    result.push(Arc::new(BenchmarkJudgment::new(result_service.clone())));
    result
}
