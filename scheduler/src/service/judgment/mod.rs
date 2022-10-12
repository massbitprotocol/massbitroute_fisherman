pub mod benchmark_judg;
pub mod http_latestblock_judg;
pub mod http_ping_judg;
//pub mod latestblock_judg;
pub mod main_judg;
//pub mod ping_judg;
pub mod websocket_judg;

use crate::persistence::services::job_result_service::JobResultService;
use async_trait::async_trait;
pub use benchmark_judg::BenchmarkJudgment;
use common::models::PlanEntity;
pub use main_judg::MainJudgment;
//pub use ping_judg::PingJudgment;
pub use websocket_judg::WebsocketJudgment;

use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

use crate::models::job_result::ProviderTask;
use crate::service::judgment::http_latestblock_judg::HttpLatestBlockJudgment;
use crate::service::judgment::http_ping_judg::HttpPingJudgment;
use common::jobs::{Job, JobResult};

use crate::service::judgment::JudgmentsResult::Failed;
use crate::service::report_portal::{ReportErrorCode, ReportFailedReasons};
use common::job_manage::JobRole;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum JudgmentsResult {
    Pass,
    Failed(ReportFailedReasons),
    Unfinished,
}

impl Display for JudgmentsResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            JudgmentsResult::Pass => {
                write!(f, "(Pass)")
            }
            Failed(reasons) => {
                write!(f, "(Failed for reason: {})", reasons.to_string())
            }
            JudgmentsResult::Unfinished => {
                write!(f, "(Unfinished)")
            }
        }
    }
}

impl JudgmentsResult {
    pub fn is_pass(&self) -> bool {
        self == &JudgmentsResult::Pass
    }
    pub fn is_failed(&self) -> bool {
        match self {
            JudgmentsResult::Failed(_) => true,
            _ => false,
        }
    }
    pub fn is_concluded(&self) -> bool {
        match self {
            JudgmentsResult::Failed(_) | JudgmentsResult::Pass => true,
            _ => false,
        }
    }
    pub fn new_failed(
        job_name: String,
        failed_detail: String,
        error_code: ReportErrorCode,
    ) -> Self {
        let reasons =
            ReportFailedReasons::new_with_single_reason(job_name, failed_detail, error_code);
        JudgmentsResult::Failed(reasons)
    }
}

#[async_trait]
pub trait ReportCheck: Sync + Send {
    fn get_name(&self) -> String;
    fn get_error_code(&self) -> ReportErrorCode;
    fn can_apply_for_result(&self, task: &ProviderTask) -> bool;
    async fn apply(
        &self,
        _plan: &PlanEntity,
        _job: &Vec<Job>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        //Todo: remove this function
        Ok(JudgmentsResult::Unfinished)
    }

    async fn apply_for_results(
        &self,
        _provider_task: &ProviderTask,
        _result: &Vec<JobResult>,
    ) -> Result<JudgmentsResult, anyhow::Error>;
}

pub fn get_report_judgments(
    config_dir: &str,
    result_service: Arc<JobResultService>,
    phase: &JobRole,
) -> Vec<Arc<dyn ReportCheck>> {
    let result: Vec<Arc<dyn ReportCheck>> = vec![
        Arc::new(BenchmarkJudgment::new(config_dir, result_service.clone())),
        Arc::new(HttpPingJudgment::new(
            config_dir,
            phase,
            result_service.clone(),
        )),
        Arc::new(HttpLatestBlockJudgment::new(
            config_dir,
            phase,
            result_service.clone(),
        )),
        Arc::new(WebsocketJudgment::new(
            config_dir,
            phase,
            result_service.clone(),
        )),
    ];
    result
}
