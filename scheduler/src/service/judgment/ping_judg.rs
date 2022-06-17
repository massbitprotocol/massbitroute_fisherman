use crate::models::tasks::ping::generator::PingConfig;
use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::{JudgmentsResult, ReportCheck};
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobDetail, JobRole};
use common::jobs::Job;
use common::models::PlanEntity;
use common::tasks::LoadConfig;
use histogram::Histogram;
use log::log;
use sea_orm::DatabaseConnection;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub struct PingJudgment {
    verification_config: PingConfig,
    regular_config: PingConfig,
    result_service: Arc<JobResultService>,
}

impl PingJudgment {
    pub fn new(config_dir: &str, result_service: Arc<JobResultService>) -> Self {
        let verification_config = PingConfig::load_config(
            format!("{}/ping.json", config_dir).as_str(),
            &JobRole::Verification,
        );
        let regular_config = PingConfig::load_config(
            format!("{}/ping.json", config_dir).as_str(),
            &JobRole::Regular,
        );
        PingJudgment {
            verification_config,
            regular_config,
            result_service,
        }
    }
}

#[async_trait]
impl ReportCheck for PingJudgment {
    fn can_apply(&self, job: &Job) -> bool {
        match job.job_name.as_str() {
            "Ping" => true,
            _ => false,
        }
    }

    async fn apply(&self, plan: &PlanEntity, job: &Job) -> Result<JudgmentsResult, Error> {
        let config = match JobRole::from_str(&*plan.phase)? {
            JobRole::Verification => &self.verification_config,
            JobRole::Regular => &self.regular_config,
        };
        self.result_service
            .get_result_pings(job.job_id.as_str())
            .await
            .and_then(|(responses, error_number)| {
                if responses.len() + (error_number as usize) < (job.repeat_number + 1) as usize {
                    // Todo: Check timeout
                    Ok(JudgmentsResult::Unfinished)
                } else {
                    let mut histogram = Histogram::new();
                    for val in responses.iter() {
                        histogram.increment(val.clone() as u64);
                    }
                    let res = histogram.percentile(config.ping_percentile);
                    log::debug!(
                        "Ping job {} has results: {:?} ans histogram {}%: {:?} ",
                        &job.job_id,
                        &responses,
                        config.ping_percentile,
                        &res
                    );
                    match res {
                        Ok(val) => {
                            if val < config.ping_response_time_threshold {
                                Ok(JudgmentsResult::Pass)
                            } else {
                                Ok(JudgmentsResult::Failed)
                            }
                        }
                        Err(err) => Ok(JudgmentsResult::Failed),
                    }
                }
            })
    }
}
