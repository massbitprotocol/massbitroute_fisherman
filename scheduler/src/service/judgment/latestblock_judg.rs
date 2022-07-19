use crate::models::job_result::ProviderTask;
use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::{JudgmentsResult, ReportCheck};
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::JobRole;
use common::jobs::Job;
use common::models::PlanEntity;
use std::sync::Arc;

#[derive(Debug)]
pub struct LatestBlockJudgment {
    verification_config: LatestBlockConfig,
    regular_config: LatestBlockConfig,
    result_service: Arc<JobResultService>,
}

impl LatestBlockJudgment {
    pub fn new(config_dir: &str, result_service: Arc<JobResultService>) -> Self {
        let verification_config = LatestBlockConfig::load_config(
            format!("{}/latest_block.json", config_dir).as_str(),
            &JobRole::Verification,
        );
        let regular_config = LatestBlockConfig::load_config(
            format!("{}/latest_block.json", config_dir).as_str(),
            &JobRole::Regular,
        );
        LatestBlockJudgment {
            verification_config,
            regular_config,
            result_service,
        }
    }
}

#[async_trait]
impl ReportCheck for LatestBlockJudgment {
    fn get_name(&self) -> String {
        String::from("LatestBlock")
    }
    fn can_apply_for_result(&self, task: &ProviderTask) -> bool {
        return task.task_name.as_str() == "LatestBlock"
            && task.task_type.as_str() == "LatestBlock";
    }

    async fn apply(&self, _plan: &PlanEntity, _job: &Vec<Job>) -> Result<JudgmentsResult, Error> {
        //Todo: Unimplement
        Ok(JudgmentsResult::Unfinished)
        /*
        let config = match JobRole::from_str(&*plan.phase)? {
            JobRole::Verification => &self.verification_config,
            JobRole::Regular => &self.regular_config,
        };

        let results = self.result_service.get_result_latest_blocks(job).await?;
        info!("Latest block results: {:?}", results);
        if results.is_empty() {
            return Ok(JudgmentsResult::Unfinished);
        }
        // Select result for judge
        let result = results
            .iter()
            .max_by(|r1, r2| r1.execution_timestamp.cmp(&r2.execution_timestamp))
            .unwrap();
        // Get late duration from execution time to block time
        if result.response.error_code != 0 {
            return Ok(JudgmentsResult::Error);
        }
        let late_duration = result.execution_timestamp - result.response.block_timestamp * 1000;
        info!(
            "execution_timestamp: {}, block_timestamp: {}, Latest block late_duration/threshold: {}s/{}s",
            result.execution_timestamp,
            result.response.block_timestamp * 1000,
            late_duration / 1000,
            config.late_duration_threshold_ms / 1000,
        );

        return if (late_duration / 1000) > (config.late_duration_threshold_ms / 1000) {
            Ok(JudgmentsResult::Failed)
        } else {
            Ok(JudgmentsResult::Pass)
        };
         */
    }
}
