use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::{JudgmentsResult, ReportCheck};
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobDetail, JobRole};
use common::jobs::Job;
use common::models::PlanEntity;
use common::tasks::eth::LatestBlockConfig;
use common::tasks::LoadConfig;
use log::{debug, info};
use minifier::js::Keyword::Default;
use sea_orm::DatabaseConnection;
use std::str::FromStr;
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
    fn can_apply(&self, job: &Job) -> bool {
        match job.job_name.as_str() {
            "LatestBlock" => true,
            _ => false,
        }
    }

    async fn apply(&self, plan: &PlanEntity, job: &Job) -> Result<JudgmentsResult, Error> {
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
    }
}
