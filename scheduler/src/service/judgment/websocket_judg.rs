use crate::models::job_result::ProviderTask;
use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::{JudgmentsResult, ReportCheck};
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::JobRole;
use common::job_manage::JobRole::Verification;
use common::jobs::Job;
use common::models::PlanEntity;
use common::tasks::eth::LatestBlockConfig;
use common::tasks::websocket_request::JobWebsocketConfig;
use common::tasks::LoadConfig;
use std::sync::Arc;

#[derive(Debug)]
pub struct WebsocketJudgment {
    job_config: JobWebsocketConfig,
    result_service: Arc<JobResultService>,
}

impl WebsocketJudgment {
    pub fn new(config_dir: &str, phase: &JobRole, result_service: Arc<JobResultService>) -> Self {
        let job_config = JobWebsocketConfig::load_config(
            format!("{}/websocket.json", config_dir).as_str(),
            phase,
        );
        WebsocketJudgment {
            job_config,
            result_service,
        }
    }
}

#[async_trait]
impl ReportCheck for WebsocketJudgment {
    fn get_name(&self) -> String {
        String::from("Websocket")
    }
    fn can_apply_for_result(&self, task: &ProviderTask) -> bool {
        return task.task_name.as_str() == "Websocket";
    }

    async fn apply(&self, _plan: &PlanEntity, _job: &Vec<Job>) -> Result<JudgmentsResult, Error> {
        //Todo: Unimplement
        Ok(JudgmentsResult::Unfinished)
    }
}
