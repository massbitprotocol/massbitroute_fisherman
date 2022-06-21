use crate::persistence::services::provider_service::ProviderService;
use crate::persistence::ProviderMapModel;
use crate::report_processors::adapters::Appender;
use anyhow::Error;
use async_trait::async_trait;
use common::tasks::ping::JobPingResult;
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
    async fn append_ping_results(&self, results: &Vec<JobPingResult>) -> Result<(), Error> {
        log::debug!("ProvidersMapAdapter append ping results");
        let provider_maps = results
            .iter()
            .map(|item| ProviderMapModel {
                id: 0,
                worker_id: item.worker_id.clone(),
                provider_id: item.job.component_id.clone(),
                ping_response_time: Some(item.response.response_time as i32),
                ping_time: None,
                bandwidth: None,
                bandwidth_time: None,
                status: Some(1),
                last_connect_time: None,
                last_check: None,
            })
            .collect::<Vec<ProviderMapModel>>();
        self.provider_service
            .store_provider_maps(&provider_maps)
            .await;
        Ok(())
    }
}
