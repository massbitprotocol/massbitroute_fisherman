use crate::models::providers::ProviderStorage;
use crate::models::workers::WorkerPool;
use crate::PORTAL_AUTHORIZATION;
use common::component::{ComponentInfo, ComponentType};
use log::debug;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

pub struct ProviderScanner {
    url_list_nodes: String,
    url_list_gateways: String,
    providers: Arc<ProviderStorage>,
    workers: Arc<WorkerPool>,
}
/*
 * Scan portal and call api to every worker to update latest status
 */
impl ProviderScanner {
    pub fn new(
        url_list_nodes: String,
        url_list_gateways: String,
        providers: Arc<ProviderStorage>,
        workers: Arc<WorkerPool>,
    ) -> Self {
        ProviderScanner {
            url_list_nodes,
            url_list_gateways,
            providers,
            workers,
        }
    }
    pub fn init(&mut self) {
        loop {
            println!("Get new provider");
            sleep(Duration::from_secs(60));
        }
    }
    pub async fn load_providers(&mut self) {}
    async fn load_nodes(&mut self) -> Result<(), anyhow::Error> {
        let res_data = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .get(self.url_list_nodes.as_str())
            .header("Authorization", PORTAL_AUTHORIZATION.as_str())
            .send()
            .await?
            .text()
            .await?;
        debug!("res_data Node: {:?}", res_data);
        let mut components: Vec<ComponentInfo> = serde_json::from_str(res_data.as_str())?;
        debug!("components Node: {:?}", components);
        for component in components.iter_mut() {
            component.component_type = ComponentType::Node;
        }
        Ok(())
    }
    async fn load_gateways(&mut self) -> Result<(), anyhow::Error> {
        //Get gateway
        let res_data = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .get(self.url_list_gateways.as_str())
            .header("Authorization", PORTAL_AUTHORIZATION.as_str())
            .send()
            .await?
            .text()
            .await?;
        debug!("res_data Gateway: {:?}", res_data);
        let mut components: Vec<ComponentInfo> = serde_json::from_str(res_data.as_str()).unwrap();
        debug!("components Gateway: {:?}", components);
        for component in components.iter_mut() {
            component.component_type = ComponentType::Gateway;
        }
        Ok(())
        /*
        //Filter components
        if let Some(status) = filter_status {
            self.list_nodes
                .retain(|component| &component.status == status);
            self.list_gateways
                .retain(|component| &component.status == status);
        }
        */
    }
}
