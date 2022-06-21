use crate::models::providers::ProviderStorage;
use crate::models::workers::WorkerInfoStorage;
use crate::persistence::services::provider_service::ProviderService;
use crate::{CONFIG, PORTAL_AUTHORIZATION};
use anyhow::Error;
use common::component::{ComponentInfo, ComponentType, Zone};
use futures_util::TryFutureExt;
use log::{debug, error, info};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::time::sleep;

pub struct ProviderScanner {
    url_list_nodes: String,
    url_list_gateways: String,
    providers: Arc<Mutex<ProviderStorage>>,
    workers: Arc<Mutex<WorkerInfoStorage>>,
    provider_service: Arc<ProviderService>,
    client: Client,
}
/*
 * Scan portal and call api to every worker to update latest status
 */
impl ProviderScanner {
    pub fn new(
        url_list_nodes: String,
        url_list_gateways: String,
        providers: Arc<Mutex<ProviderStorage>>,
        workers: Arc<Mutex<WorkerInfoStorage>>,
        provider_service: Arc<ProviderService>,
    ) -> Self {
        ProviderScanner {
            url_list_nodes,
            url_list_gateways,
            providers,
            workers,
            provider_service,
            client: {
                reqwest::Client::builder()
                    .danger_accept_invalid_certs(true)
                    .build()
                    .unwrap()
            },
        }
    }

    pub async fn run(mut self) {
        loop {
            //info!("Get new providers");
            self.update_providers().await;
            //Update provider map
            self.get_provider_map().await;
            sleep(Duration::from_secs(CONFIG.update_provider_list_interval)).await;
        }
    }

    pub async fn update_providers(&mut self) -> Result<(), Error> {
        let nodes = self
            .get_components_list(ComponentType::Node, Some("staked"), &Zone::GB, None)
            .await;
        let gateways = self
            .get_components_list(ComponentType::Gateway, Some("staked"), &Zone::GB, None)
            .await;
        let mut res = Ok(());
        {
            let mut lock = self.providers.lock().await;
            match nodes {
                Ok(nodes) => {
                    debug!("Found {} Nodes.", nodes.len());
                    lock.update_components_list(ComponentType::Node, nodes)
                        .await;
                }
                Err(err) => {
                    error!("Cannot get list node: {}", err);
                    res = Err(err);
                }
            }
            match gateways {
                Ok(gateways) => {
                    debug!("Found {} Gateways.", gateways.len());
                    lock.update_components_list(ComponentType::Gateway, gateways)
                        .await;
                }
                Err(err) => {
                    error!("Cannot get list gateways: {}", err);
                    res = Err(err);
                }
            }
        }
        res
    }

    pub async fn get_components_list(
        &self,
        component_type: ComponentType,
        filter_status: Option<&str>,
        filter_zone: &Zone,
        filter_chain_id: Option<&str>,
    ) -> Result<Vec<ComponentInfo>, anyhow::Error> {
        // Get nodes
        let url = match component_type {
            ComponentType::Node => &self.url_list_nodes,
            ComponentType::Gateway => &self.url_list_gateways,
        };

        debug!("list component url:{}", url);
        let res_data = self
            .client
            .get(url)
            .header("Authorization", PORTAL_AUTHORIZATION.as_str())
            .send()
            .await?
            .text()
            .await?;
        //debug!("res_data Node: {:?}", res_data);
        let mut components: Vec<ComponentInfo> = serde_json::from_str(res_data.as_str())?;
        //debug!("components Node: {:?}", components);
        // Add component type because Portal did not return the info
        for component in components.iter_mut() {
            component.component_type = component_type.clone();
        }

        //Filter components
        if let Some(status) = filter_status {
            components.retain(|component| &component.status == status);
        }

        //Filter zone
        //info!("Zone:{:?}", filter_zone);
        if *filter_zone != Zone::GB {
            components.retain(|component| component.zone == *filter_zone);
        }

        //Filter ChainId
        if let Some(chain_id) = filter_chain_id {
            components.retain(|component| component.get_chain_id() == *chain_id);
        }

        Ok(components)
    }
    /*
     * Get provider map from database
     */
    pub async fn get_provider_map(&mut self) {
        let map = self.provider_service.get_provider_maps().await;
        if map.len() > 0 {
            self.workers.lock().await.set_map_worker_provider(map);
        }
    }
}
