use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::ReportCheck;
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{Job, JobDetail};
use common::models::PlanEntity;
use histogram::Histogram;
use log::log;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct PingJudgment {
    result_service: Arc<JobResultService>,
}

impl PingJudgment {
    pub fn new(result_service: Arc<JobResultService>) -> Self {
        PingJudgment { result_service }
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

    async fn apply(&self, plan: &PlanEntity, job: &Job) -> Result<u32, Error> {
        if let Ok(responses) = self
            .result_service
            .get_result_pings(plan.plan_id.as_str())
            .await
        {
            for (worker_id, response_times) in responses.iter() {
                let mut histogram = Histogram::new();
                for val in response_times {
                    histogram.increment(val.clone() as u64);
                }
                let res = histogram.percentile(50_f64);
                log::debug!("Worker {:?}, histogram: {:?}", worker_id, &res);
            }
            log::debug!("Ping results: {:?}", &responses);
        }
        Ok(0)
    }
}
