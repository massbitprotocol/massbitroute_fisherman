use crate::models::job_result::ProviderTask;
use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::{JudgmentsResult, ReportCheck};
use crate::tasks::benchmark::generator::BenchmarkConfig;
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobDetail, JobRole};
use common::jobs::{Job, JobResult};
use common::models::PlanEntity;
use common::tasks::LoadConfig;
use common::Timestamp;
use log::info;
use sea_orm::DatabaseConnection;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub struct BenchmarkJudgment {
    result_service: Arc<JobResultService>,
    verification_config: BenchmarkConfig,
    regular_config: BenchmarkConfig,
}

impl BenchmarkJudgment {
    pub fn new(config_dir: &str, result_service: Arc<JobResultService>) -> Self {
        let verification_config = BenchmarkConfig::load_config(
            format!("{}/benchmark.json", config_dir).as_str(),
            &JobRole::Verification,
        );
        let regular_config = BenchmarkConfig::load_config(
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
        let config = match JobRole::from_str(&*plan.phase)? {
            JobRole::Verification => &self.verification_config,
            JobRole::Regular => &self.regular_config,
        };

        let results = self.result_service.get_result_benchmarks(job).await?;

        info!("Benchmark results: {:?}", results);
        if results.is_empty() {
            return Ok(JudgmentsResult::Unfinished);
        }

        // Select result for judge
        let result = results
            .iter()
            .max_by(|r1, r2| r1.response_timestamp.cmp(&r2.response_timestamp))
            .unwrap();
        // Fixme: histogram config is dynamic.
        let result_response_time = result
            .response
            .histograms
            .get(&config.judge_histogram_percentile);

        info!("result_response_time: {:?}", result_response_time);

        return match result_response_time {
            None => Ok(JudgmentsResult::Error),
            Some(result_response_time) => {
                info!(
                    "Check result_response_time: {}, response_threshold: {}",
                    result_response_time, config.response_threshold
                );
                if (*result_response_time as Timestamp) > config.response_threshold {
                    Ok(JudgmentsResult::Failed)
                } else {
                    Ok(JudgmentsResult::Pass)
                }
            }
        };
    }
    async fn apply_for_results(
        &self,
        provider_task: &ProviderTask,
        result: &Vec<JobResult>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        Ok(JudgmentsResult::Error)
    }
}
