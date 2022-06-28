use crate::models::job_result::ProviderTask;
use crate::persistence::services::JobResultService;
use crate::service::judgment::{JudgmentsResult, ReportCheck};
use crate::tasks::ping::generator::PingConfig;
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobResultDetail, JobRole};
use common::jobs::{Job, JobResult};
use common::models::PlanEntity;
use common::tasks::http_request::{JobHttpResponseDetail, JobHttpResult};
use common::tasks::LoadConfig;
use common::Timestamp;
use histogram::Histogram;
use log::info;
use std::collections::HashMap;
use std::default::Default;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct HttpPingResultCache {
    response_times: Mutex<HashMap<ProviderTask, JudRoundTripTimeDatas>>,
}

#[derive(Debug, Default, Clone)]
pub struct JudRoundTripTimeDatas {
    inner: Vec<JudRoundTripTimeData>,
}

impl JudRoundTripTimeDatas {
    fn new() -> Self {
        JudRoundTripTimeDatas { inner: vec![] }
    }
    fn get_success_percent(&self) -> f64 {
        let success_count = self.iter().filter(|&n| n.success == true).count() as f64;
        success_count * 100.0 / (self.len() as f64)
    }
}

impl Deref for JudRoundTripTimeDatas {
    type Target = Vec<JudRoundTripTimeData>;

    fn deref(&self) -> &Vec<JudRoundTripTimeData> {
        &self.inner
    }
}
impl DerefMut for JudRoundTripTimeDatas {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug, Default, Clone)]
pub struct JudRoundTripTimeData {
    response_time: Timestamp,
    receive_timestamp: Timestamp,
    success: bool,
}

impl JudRoundTripTimeData {
    fn new_false(receive_time: Timestamp) -> Self {
        JudRoundTripTimeData {
            receive_timestamp: receive_time,
            success: false,
            response_time: Default::default(),
        }
    }
}

impl HttpPingResultCache {
    pub async fn append_results(
        &self,
        provider_task: &ProviderTask,
        results: &Vec<JobResult>,
    ) -> JudRoundTripTimeDatas {
        let mut res_times = JudRoundTripTimeDatas::new();
        for res in results.iter() {
            if let JobResultDetail::HttpRequest(JobHttpResult { response, .. }) = &res.result_detail
            {
                let mut data = JudRoundTripTimeData::new_false(res.receive_timestamp);
                if let JobHttpResponseDetail::Body(val) = &response.detail {
                    if let Ok(response_time) = val.parse::<Timestamp>() {
                        data = JudRoundTripTimeData {
                            response_time,
                            receive_timestamp: res.receive_timestamp,
                            success: true,
                        };
                    }
                };
                res_times.push(data);
            }
        }
        let mut values = self.response_times.lock().await;
        let values = values
            .entry(provider_task.clone())
            .or_insert(JudRoundTripTimeDatas::new());
        values.append(&mut res_times);
        res_times = values.clone();
        res_times
    }
}
#[derive(Debug)]
pub struct HttpPingJudgment {
    verification_config: PingConfig,
    regular_config: PingConfig,
    result_service: Arc<JobResultService>,
    result_cache: HttpPingResultCache,
}

impl HttpPingJudgment {
    pub fn new(config_dir: &str, result_service: Arc<JobResultService>) -> Self {
        let verification_config = PingConfig::load_config(
            format!("{}/ping.json", config_dir).as_str(),
            &JobRole::Verification,
        );
        let regular_config = PingConfig::load_config(
            format!("{}/ping.json", config_dir).as_str(),
            &JobRole::Regular,
        );
        HttpPingJudgment {
            verification_config,
            regular_config,
            result_service,
            result_cache: HttpPingResultCache::default(),
        }
    }
}

#[async_trait]
impl ReportCheck for HttpPingJudgment {
    fn can_apply(&self, job: &Job) -> bool {
        match job.job_name.as_str() {
            "RoundTripTime" => true,
            _ => false,
        }
    }
    fn can_apply_for_result(&self, task: &ProviderTask) -> bool {
        return task.task_type.as_str() == "HttpRequest"
            && task.task_name.as_str() == "RoundTripTime";
    }

    async fn apply(&self, plan: &PlanEntity, job: &Job) -> Result<JudgmentsResult, Error> {
        todo!();
    }

    async fn apply_for_results(
        &self,
        provider_task: &ProviderTask,
        result: &Vec<JobResult>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        if result.is_empty() {
            return Ok(JudgmentsResult::Unfinished);
        }
        let phase = result.first().unwrap().phase.clone();
        let response_times = self
            .result_cache
            .append_results(provider_task, result)
            .await;
        let config = match phase {
            JobRole::Verification => &self.verification_config,
            JobRole::Regular => &self.regular_config,
        };

        info!(
            "{} Http Ping cache: {:?}",
            response_times.len(),
            response_times
        );
        return if response_times.len() < config.ping_number_for_decide as usize {
            // Todo: Check timeout
            Ok(JudgmentsResult::Unfinished)
        } else if response_times.get_success_percent() < config.ping_success_percent_threshold {
            Ok(JudgmentsResult::Error)
        } else {
            let mut histogram = Histogram::new();
            for val in response_times.iter() {
                if val.success {
                    histogram.increment(val.response_time as u64);
                }
            }

            let res = histogram.percentile(config.ping_percentile);
            log::debug!(
                "Http Ping job on {} has results: {:?} ans histogram {}%: {:?} ",
                &provider_task.provider_id,
                &response_times,
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
        };
    }
}
