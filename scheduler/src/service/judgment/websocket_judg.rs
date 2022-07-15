use crate::models::job_result::ProviderTask;
use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::{JudgmentsResult, ReportCheck};
use anyhow::anyhow;
use async_trait::async_trait;
use common::job_manage::{JobResultDetail, JobRole};
use common::jobs::JobResult;
use common::tasks::websocket_request::JobWebsocketConfig;
use std::sync::Arc;

#[derive(Debug)]
pub struct WebsocketJudgment {
    job_configs: Vec<JobWebsocketConfig>,
    result_service: Arc<JobResultService>,
}

impl WebsocketJudgment {
    pub fn new(config_dir: &str, phase: &JobRole, result_service: Arc<JobResultService>) -> Self {
        let job_configs = JobWebsocketConfig::read_config(
            format!("{}/websocket.json", config_dir).as_str(),
            phase,
        );
        WebsocketJudgment {
            job_configs,
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
        return task.task_type.as_str() == "Websocket";
    }

    async fn apply_for_results(
        &self,
        _provider_task: &ProviderTask,
        job_results: &Vec<JobResult>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        if job_results.is_empty() {
            return Ok(JudgmentsResult::Unfinished);
        }
        // Get comparator for the fist item
        let first_result = job_results.first().unwrap();
        log::debug!("{:?}", &first_result);
        //For websocket only check if worker can connect to provider and get data
        if let JobResultDetail::Websocket(web_socket_result) = &first_result.result_detail {
            Ok(JudgmentsResult::Pass)
        } else {
            Ok(JudgmentsResult::Failed)
        }
    }
}
