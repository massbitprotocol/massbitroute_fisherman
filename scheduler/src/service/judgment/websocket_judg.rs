use crate::models::job_result::ProviderTask;
use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::{JudgmentsResult, ReportCheck};
use std::path::Path;

use crate::models::job_result_cache::TaskName;
use crate::service::report_portal::ReportErrorCode;
use crate::CONFIG_WEBSOCKET_DIR;
use async_trait::async_trait;
use common::job_manage::{JobResultDetail, JobRole};
use common::jobs::JobResult;
use common::tasks::websocket_request::{JobWebsocketConfig, JobWebsocketResponseDetail};
use common::tasks::LoadConfigs;
use log::info;
use std::sync::Arc;

#[derive(Debug)]
pub struct WebsocketJudgment {
    _job_configs: Vec<JobWebsocketConfig>,
    _result_service: Arc<JobResultService>,
}

impl WebsocketJudgment {
    pub fn new(config_dir: &str, phase: &JobRole, result_service: Arc<JobResultService>) -> Self {
        // let job_configs = JobWebsocketConfig::read_config(
        //     format!("{}/websocket.json", config_dir).as_str(),
        //     phase,
        // );
        //let path = format!("{}/websocket", config_dir);
        let path = Path::new(config_dir).join(&*CONFIG_WEBSOCKET_DIR);
        let job_configs = JobWebsocketConfig::read_configs(&path, phase);
        WebsocketJudgment {
            _job_configs: job_configs,
            _result_service: result_service,
        }
    }
    pub fn get_config(&self, name: &TaskName) -> Option<&JobWebsocketConfig> {
        for config in &self._job_configs {
            if &config.name == name {
                info!("&config.name: {}, &config.name: {}", config.name, name);
                return Some(config);
            }
        }
        None
    }
}

#[async_trait]
impl ReportCheck for WebsocketJudgment {
    fn get_name(&self) -> String {
        String::from("Websocket")
    }
    fn get_error_code(&self) -> ReportErrorCode {
        ReportErrorCode::WebsocketCallFailed
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
        let config = self.get_config(&first_result.job_name);
        if config.is_none() {
            let failed_reason = format!(
                "Error: Cannot load config from {:?} for {:?}",
                self._job_configs, &first_result.job_name
            );
            return Ok(JudgmentsResult::new_failed(
                self.get_name(),
                failed_reason,
                ReportErrorCode::WebsocketJudgementFailed,
            ));
        }
        let config = config.unwrap();

        log::debug!("apply_for_results results websocket: {:?}", &first_result);
        log::debug!(
            "apply_for_results _provider_task websocket: {:?}",
            &_provider_task
        );

        //For websocket only check if worker can connect to provider and get data
        if let JobResultDetail::Websocket(web_socket_result) = &first_result.result_detail {
            if web_socket_result.error_code == 0 {
                if let JobWebsocketResponseDetail::Values(value) = &web_socket_result.detail {
                    // Check require field exist:
                    info!("apply_for_results config: {:?}", config);
                    for (key, _value) in &config.response.values {
                        if !value.contains_key(key) {
                            let failed_reason =
                                format!("Response do not contain: {} in: {:?}", key, value);

                            return Ok(JudgmentsResult::new_failed(
                                self.get_name(),
                                failed_reason,
                                ReportErrorCode::WebsocketCallFailed,
                            ));
                        }
                    }

                    return Ok(JudgmentsResult::Pass);
                } else {
                    let failed_reason = format!(
                        "Error: Result detail is not a hashmap, result_detail: {:?}",
                        first_result.result_detail
                    );
                    Ok(JudgmentsResult::new_failed(
                        self.get_name(),
                        failed_reason,
                        ReportErrorCode::WebsocketCallFailed,
                    ))
                }
            } else {
                let failed_reason = format!(
                    "error_code: {}, message: {}",
                    web_socket_result.error_code, web_socket_result.message
                );
                Ok(JudgmentsResult::new_failed(
                    self.get_name(),
                    failed_reason,
                    ReportErrorCode::WebsocketCallFailed,
                ))
            }
        } else {
            let failed_reason = format!(
                "Result type should be Websocket instead {:?}",
                first_result.result_detail
            );
            Ok(JudgmentsResult::new_failed(
                self.get_name(),
                failed_reason,
                ReportErrorCode::WebsocketJudgementFailed,
            ))
        }
    }
}
