use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::{JudgmentsResult, ReportCheck};
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::JobDetail;
use common::jobs::Job;
use common::models::PlanEntity;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Debug)]
pub struct BenchmarkJudgment {
    result_service: Arc<JobResultService>,
    verification_config: LatestBenchmarkConfig,
    regular_config: LatestBenchmarkConfig,
}

impl BenchmarkJudgment {
    pub fn new(config_dir: &str, result_service: Arc<JobResultService>) -> Self {
        let verification_config = LatestBenchmarkConfig::load_config(
            format!("{}/benchmark.json", config_dir).as_str(),
            &JobRole::Verification,
        );
        let regular_config = LatestBenchmarkConfig::load_config(
            format!("{}/benchmark.json", config_dir).as_str(),
            &JobRole::Regular,
        );
        BenchmarkJudgment {
            result_service,
            verification_config,
            regular_config,
        }
    }
}

#[async_trait]
impl ReportCheck for BenchmarkJudgment {
    fn can_apply(&self, job: &Job) -> bool {
        match job.job_name.as_str() {
            "Benchmark" => true,
            _ => false,
        }
    }

    async fn apply(&self, plan: &PlanEntity, job: &Job) -> Result<JudgmentsResult, Error> {
        Ok(JudgmentsResult::Pass)
    }
}
