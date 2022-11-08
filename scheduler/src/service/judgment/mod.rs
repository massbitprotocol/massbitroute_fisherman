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
use std::fmt::Debug;

use crate::models::job_result::ProviderTask;
use crate::service::judgment::http_latestblock_judg::HttpLatestBlockJudgment;
use crate::service::judgment::http_ping_judg::HttpPingJudgment;
use common::jobs::{Job, JobResult};

use common::job_manage::JobRole;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
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
        self == &JudgmentsResult::Failed || self == &JudgmentsResult::Error
    }
    pub fn is_concluded(&self) -> bool {
        self == &JudgmentsResult::Pass
            || self == &JudgmentsResult::Failed
            || self == &JudgmentsResult::Error
    }
}

#[async_trait]
pub trait ReportCheck: Sync + Send {
    fn get_name(&self) -> String;
    //fn can_apply(&self, job: &Job) -> bool;
    fn can_apply_for_result(&self, task: &ProviderTask) -> bool;
    async fn apply(
        &self,
        _plan: &PlanEntity,
        _job: &[Job],
    ) -> Result<JudgmentsResult, anyhow::Error> {
        //Todo: remove this function
        Ok(JudgmentsResult::Unfinished)
    }

    async fn apply_for_results(
        &self,
        _provider_task: &ProviderTask,
        _result: &[JobResult],
    ) -> Result<JudgmentsResult, anyhow::Error> {
        Ok(JudgmentsResult::Error)
    }
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
        Arc::new(WebsocketJudgment::new(config_dir, phase, result_service)),
    ];
    result
}
