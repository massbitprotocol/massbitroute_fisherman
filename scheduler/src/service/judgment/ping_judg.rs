use crate::models::tasks::ping::generator::PingConfig;
use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::ReportCheck;
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobDetail, JobRole};
use common::jobs::Job;
use common::models::PlanEntity;
use common::tasks::LoadConfig;
use histogram::Histogram;
use log::log;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

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
        match job.job_detail {
            Some(JobDetail::Ping(_)) => true,
            _ => false,
        }
    }

    async fn apply(&self, plan: &PlanEntity, job: &Job) -> Result<i32, Error> {
        self.result_service
            .get_result_pings(job.job_id.as_str())
            .await
            .and_then(|responses| {
                if responses.len() < (job.repeat_number + 1) as usize {
                    Ok(0)
                } else {
                    let mut histogram = Histogram::new();
                    for val in responses.iter() {
                        histogram.increment(val.clone() as u64);
                    }
                    let res = histogram.percentile(50_f64);
                    log::debug!(
                        "Ping job {} has results: {:?} ans histogram 99 {:?} ",
                        &job.job_id,
                        &responses,
                        &res
                    );
                    match res {
                        Ok(val) => {
                            if val < 300 {
                                Ok(1)
                            } else {
                                Ok(-1)
                            }
                        }
                        Err(err) => Ok(-1),
                    }
                }
            })
    }
}
