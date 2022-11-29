use crate::models::job_result::ProviderTask;
use crate::persistence::services::JobResultService;
use crate::service::judgment::{JudgmentsResult, ReportCheck};

use crate::service::report_portal::ReportErrorCode;
use crate::CONFIG_HTTP_REQUEST_DIR;
use anyhow::anyhow;
use async_trait::async_trait;
use common::job_manage::{JobResultDetail, JobRole};
use common::jobs::{AssignmentConfig, JobResult};
use common::tasks::http_request::{HttpRequestJobConfig, JobHttpResponseDetail, JobHttpResult};
use common::tasks::{LoadConfigs, TaskConfigTrait};
use common::util::warning_if_error;
use common::Timestamp;
use histogram::Histogram;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::default::Default;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct PingConfig {
    pub ping_success_percent_threshold: f64, //
    pub ping_percentile: f64,
    pub ping_response_time_threshold: u64,
    pub repeat_number: i32,
    pub ping_request_response: String,
    pub ping_timeout_ms: Timestamp,
    pub ping_number_for_decide: i32,
    pub assignment: Option<AssignmentConfig>,
}

#[derive(Debug, Default)]
pub struct HttpPingResultCache {
    response_durations: Mutex<HashMap<ProviderTask, JudRoundTripTimeDatas>>,
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
        let success_count = self.iter().filter(|&data| data.success).count() as f64;
        success_count * 100.0 / (self.len() as f64)
    }
    fn get_response_messages(&self) -> String {
        self.iter()
            .filter(|data| data.response_message.len() > 0)
            .map(|data| data.response_message.clone())
            .collect::<Vec<String>>()
            .join(",")
    }
    /*
     * Tai: 2022-11-29
     * If only push result into the cache then it take to long to reach the failed ratio
     */
    fn append(&mut self, other: &mut Self, number_for_decide: usize) {
        while self.inner.len() > number_for_decide {
            self.inner.remove(0);
        }
        self.inner.append(other);
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
    response_duration: Timestamp,
    _receive_timestamp: Timestamp,
    success: bool,
    response_message: String,
}

impl JudRoundTripTimeData {
    fn new(receive_time: Timestamp) -> Self {
        JudRoundTripTimeData {
            _receive_timestamp: receive_time,
            success: false,
            response_duration: Timestamp::default(),
            response_message: String::default(),
        }
    }
}

impl HttpPingResultCache {
    pub async fn append_results(
        &self,
        provider_task: &ProviderTask,
        results: &Vec<JobResult>,
        number_for_decide: usize,
    ) -> JudRoundTripTimeDatas {
        let mut res_times = JudRoundTripTimeDatas::new();
        for res in results.iter() {
            if let JobResultDetail::HttpRequest(JobHttpResult { response, .. }) = &res.result_detail
            {
                //Default success if false
                let mut data = JudRoundTripTimeData::new(res.receive_timestamp);
                trace!("append_results response: {:?}", response);
                if let JobHttpResponseDetail::Body(val) = &response.detail {
                    if let Ok(response_duration) = val.parse::<Timestamp>() {
                        // Change unit of RTT response from us -> ms
                        data.response_duration = response_duration / 1000;
                        data.success = true;
                        data.response_message = response.message.clone();
                    }
                };
                res_times.push(data);
            }
        }
        {
            let mut values = self.response_durations.lock().await;
            let values = values
                .entry(provider_task.clone())
                .or_insert(JudRoundTripTimeDatas::new());
            values.append(&mut res_times, number_for_decide);
            res_times = values.clone();
        }
        res_times
    }
}
#[derive(Debug)]
pub struct HttpPingJudgment {
    task_configs: Vec<HttpRequestJobConfig>,
    _result_service: Arc<JobResultService>,
    result_cache: HttpPingResultCache,
}

impl HttpPingJudgment {
    pub fn new(config_dir: &str, phase: &JobRole, result_service: Arc<JobResultService>) -> Self {
        // let path = format!("{}/http_request", config_dir);
        let path = Path::new(config_dir).join(&*CONFIG_HTTP_REQUEST_DIR);
        //let task_configs = HttpRequestJobConfig::read_config(path.as_str(), phase);
        let task_configs = HttpRequestJobConfig::read_configs(&path, phase)
            .into_iter()
            .filter(|config| {
                config.name.as_str() == "RoundTripTime" || config.name.as_str() == "Ping"
            })
            .collect::<Vec<HttpRequestJobConfig>>();
        HttpPingJudgment {
            task_configs,
            _result_service: result_service,
            result_cache: HttpPingResultCache::default(),
        }
    }
    pub fn get_judgment_thresholds(&self, phase: &JobRole) -> Map<String, Value> {
        trace!(
            "get_judgment_thresholds task_configs: {:#?}",
            self.task_configs
        );
        self.task_configs
            .iter()
            .filter(|config| config.match_phase(phase))
            .collect::<Vec<&HttpRequestJobConfig>>()
            .get(0)
            .map(|config| config.thresholds.clone())
            .unwrap_or_default()
    }
    pub fn get_threshold_value(
        thresholds: &Map<String, Value>,
        field: &String,
    ) -> Result<i64, anyhow::Error> {
        thresholds
            .get(field)
            .ok_or(anyhow!("Missing threshold config {}", field))?
            .as_i64()
            .ok_or(anyhow!("Invalid threshold config {}", field))
    }
}

#[async_trait]
impl ReportCheck for HttpPingJudgment {
    fn get_name(&self) -> String {
        String::from("HttpPing")
    }
    fn can_apply_for_result(&self, task: &ProviderTask) -> bool {
        return task.task_type.as_str() == "HttpRequest"
            && task.task_name.as_str() == "RoundTripTime";
    }
    fn get_error_code(&self) -> ReportErrorCode {
        ReportErrorCode::RoundTripTimeCallFailed
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
        // Get threshold from config
        let thresholds = self.get_judgment_thresholds(&phase);
        trace!(
            "apply_for_results thresholds phase {}: {:?}",
            phase.to_string(),
            thresholds
        );
        let number_for_decide =
            Self::get_threshold_value(&thresholds, &String::from("number_for_decide"))?;
        let success_percent_threshold =
            Self::get_threshold_value(&thresholds, &String::from("success_percent"))?;
        let histogram_percentile_threshold =
            Self::get_threshold_value(&thresholds, &String::from("histogram_percentile"))?;
        let response_duration_threshold =
            Self::get_threshold_value(&thresholds, &String::from("response_duration"))?;
        let response_durations = self
            .result_cache
            .append_results(provider_task, result, number_for_decide as usize)
            .await;
        debug!("{} Http Ping in cache.", response_durations.len());
        return if response_durations.len() < number_for_decide as usize {
            Ok(JudgmentsResult::Unfinished)
        } else if response_durations.get_success_percent() < success_percent_threshold as f64 {
            let failed_reason = format!(
                "Success percent {} < {success_percent_threshold}. Detail messages {}",
                response_durations.get_success_percent(),
                response_durations.get_response_messages()
            );
            Ok(JudgmentsResult::new_failed(
                self.get_name(),
                failed_reason,
                ReportErrorCode::RoundTripTimeSuccessPercentFailed,
            ))
        } else {
            let mut histogram = Histogram::new();
            for val in response_durations.iter() {
                if val.success {
                    let res = histogram
                        .increment(val.response_duration as u64)
                        .map_err(|e| anyhow!("Error: {}", e));
                    warning_if_error("histogram.increment return error", res);
                }
            }

            let res = histogram.percentile(histogram_percentile_threshold as f64);
            log::trace!(
                "Http Ping job on {} has results: {:?} ans histogram {}%: {:?} ",
                &provider_task.provider_id,
                &response_durations,
                histogram_percentile_threshold,
                &res
            );

            match res {
                Ok(val) => {
                    if val <= response_duration_threshold as u64 {
                        Ok(JudgmentsResult::Pass)
                    } else {
                        let failed_reason =
                            format!("{histogram_percentile_threshold}% response duration : {val} > {response_duration_threshold}");
                        Ok(JudgmentsResult::new_failed(
                            self.get_name(),
                            failed_reason,
                            ReportErrorCode::RoundTripTimeResponseTimeFailed,
                        ))
                    }
                }
                Err(err) => {
                    let failed_reason = format!("Cannot get response duration, with error: {err}");
                    Ok(JudgmentsResult::new_failed(
                        self.get_name(),
                        failed_reason,
                        ReportErrorCode::RoundTripTimeCallFailed,
                    ))
                }
            }
        };
    }
}
