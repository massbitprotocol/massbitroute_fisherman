pub mod benchmark_judg;
pub mod main_judg;
pub mod ping_judg;

pub use benchmark_judg::BenchmarkJudgment;
pub use ping_judg::PingJudgment;

use crate::persistence::services::job_result_service::JobResultService;
use async_trait::async_trait;
pub use main_judg::Judgment;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[async_trait]
pub trait ReportCheck: Sync + Send {
    fn can_apply(&self) -> bool;
    /*
     * result >= 0 result is looked good
     * result < 0 something is bad
     */
    async fn apply(&self) -> Result<u32, anyhow::Error>;
}

pub fn get_report_judgments(result_service: Arc<JobResultService>) -> Vec<Arc<dyn ReportCheck>> {
    let mut result: Vec<Arc<dyn ReportCheck>> = Default::default();
    result.push(Arc::new(PingJudgment::new(result_service.clone())));
    result.push(Arc::new(BenchmarkJudgment::new(result_service.clone())));
    result
}