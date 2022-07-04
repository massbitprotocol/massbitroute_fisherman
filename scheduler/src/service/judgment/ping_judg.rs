use crate::models::job_result::ProviderTask;
use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::{JudgmentsResult, ReportCheck};
use crate::tasks::ping::generator::PingConfig;
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobDetail, JobResultDetail, JobRole};
use common::jobs::{Job, JobResult};
use common::models::PlanEntity;
use common::tasks::http_request::{JobHttpResponse, JobHttpResponseDetail, JobHttpResult};
use common::tasks::LoadConfig;
use histogram::Histogram;
use log::log;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct PingResultCache {
    response_times: Mutex<HashMap<ProviderTask, Vec<u64>>>,
}

impl PingResultCache {
    pub fn append_results(
        &self,
        provider_task: &ProviderTask,
        results: &Vec<JobResult>,
    ) -> Vec<u64> {
        let mut res_times = Vec::new();
        for res in results.iter() {
            if let JobResultDetail::HttpRequest(JobHttpResult { response, .. }) = &res.result_detail
            {
                if let JobHttpResponseDetail::Body(val) = &response.detail {
                    if let Ok(time) = val.parse::<u64>() {
                        res_times.push(time);
                    }
                }
            }
        }
        let mut values = self.response_times.lock().unwrap();
        let values = values.entry(provider_task.clone()).or_insert(Vec::new());
        values.append(&mut res_times);
        res_times = values.clone();
        res_times
    }
}
#[derive(Debug)]
pub struct PingJudgment {
    verification_config: PingConfig,
    regular_config: PingConfig,
    result_service: Arc<JobResultService>,
    result_cache: PingResultCache,
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
            result_cache: PingResultCache::default(),
        }
    }
}

#[async_trait]
impl ReportCheck for PingJudgment {
    fn get_name(&self) -> String {
        String::from("Ping")
    }
    fn can_apply(&self, job: &Job) -> bool {
        match job.job_name.as_str() {
            "Ping" => true,
            _ => false,
        }
    }

    async fn apply(&self, plan: &PlanEntity, job: &Vec<Job>) -> Result<JudgmentsResult, Error> {
        Ok(JudgmentsResult::Unfinished)
        /*
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

         */
    }
    async fn apply_for_results(
        &self,
        provider_task: &ProviderTask,
        result: &Vec<JobResult>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        let response_times = self.result_cache.append_results(provider_task, result);
        Ok(JudgmentsResult::Error)
    }
}
