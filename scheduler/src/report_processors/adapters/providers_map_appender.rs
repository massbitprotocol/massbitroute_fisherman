use crate::persistence::services::provider_service::ProviderService;
use crate::persistence::ProviderMapModel;
use crate::report_processors::adapters::Appender;
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::JobResultDetail;
use common::jobs::JobResult;
use common::tasks::http_request::{JobHttpResponseDetail, JobHttpResult};
use common::tasks::ping::JobPingResult;
use common::util::{get_current_time, remove_break_line, warning_if_error};
use log::debug;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct ProvidersMapAdapter {
    provider_service: ProviderService,
}

impl ProvidersMapAdapter {
    pub fn new(connection: Arc<DatabaseConnection>) -> Self {
        let provider_service = ProviderService::new(connection);
        ProvidersMapAdapter { provider_service }
    }
}

#[async_trait]
impl Appender for ProvidersMapAdapter {
    fn get_name(&self) -> String {
        "ProvidersMapAdapter".to_string()
    }
    async fn append_job_results(&self, results: &Vec<JobResult>) -> Result<(), anyhow::Error> {
        log::debug!("ProvidersMapAdapter append RoundTripTime results");
        let current_time = get_current_time() as i64;
        let provider_maps = results
            .iter()
            .filter(|item| {
                item.result_detail.get_name().as_str() == "HttpRequest"
                    && item.job_name.as_str() == "RoundTripTime"
            })
            .map(|item| {
                let (rtt, ping_timestamp) = match &item.result_detail {
                    JobResultDetail::HttpRequest(JobHttpResult { response, .. }) => {
                        match &response.detail {
                            JobHttpResponseDetail::Body(val) => {
                                let parsed_value = remove_break_line(val)
                                    .parse::<i32>()
                                    .map(|val| val / 1000)
                                    .ok();
                                debug!("Round trip time response {:?}, {:?}", val, &parsed_value);
                                (parsed_value, Some(response.request_timestamp))
                            }
                            JobHttpResponseDetail::Values(_) => (None, None),
                        }
                    }
                    _ => (None, None),
                };

                ProviderMapModel {
                    id: 0,
                    worker_id: item.worker_id.clone(),
                    provider_id: item.provider_id.clone(),
                    ping_response_duration: rtt,
                    ping_timestamp,
                    bandwidth: None,
                    bandwidth_timestamp: None,
                    status: Some(1),
                    last_connect_time: Some(current_time.clone()),
                    last_check: Some(current_time),
                }
            })
            .collect::<Vec<ProviderMapModel>>();
        if provider_maps.len() > 0 {
            let res = self
                .provider_service
                .store_provider_maps(&provider_maps)
                .await;
            warning_if_error("store_provider_maps return error", res);
        }
        Ok(())
    }
    async fn append_ping_results(&self, results: &Vec<JobPingResult>) -> Result<(), Error> {
        log::debug!("ProvidersMapAdapter append ping results");
        let current_time = get_current_time() as i64;
        let provider_maps = results
            .iter()
            .map(|result| ProviderMapModel {
                id: 0,
                worker_id: result.worker_id.clone(),
                provider_id: result.job.component_id.clone(),
                ping_response_duration: Some(result.response.response_duration as i32),
                ping_timestamp: None,
                bandwidth: None,
                bandwidth_timestamp: None,
                status: Some(1),
                last_connect_time: Some(current_time.clone()),
                last_check: Some(current_time),
            })
            .collect::<Vec<ProviderMapModel>>();
        let res = self
            .provider_service
            .store_provider_maps(&provider_maps)
            .await;
        warning_if_error("store_provider_maps return error", res);
        Ok(())
    }
}
