use crate::models::providers::ProviderStorage;
use crate::models::workers::WorkerInfoStorage;
use crate::persistence::services::provider_service::ProviderService;
use crate::{CONFIG, PORTAL_AUTHORIZATION};
use anyhow::Error;
use common::component::{ComponentInfo, ComponentType, Zone};
use log::{debug, error, trace};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

use tokio::time::sleep;

pub struct ProviderScanner {
    url_list_nodes: String,
    url_list_gateways: String,
    providers: Arc<ProviderStorage>,
    workers: Arc<WorkerInfoStorage>,
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
        providers: Arc<ProviderStorage>,
        workers: Arc<WorkerInfoStorage>,
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
            debug!("Get new providers");
            self.update_providers().await;
            //Update provider map
            self.reload_provider_map().await;
            debug!("Sleep for {} seconds", CONFIG.update_provider_list_interval);
            sleep(Duration::from_secs(
                CONFIG.update_provider_list_interval as u64,
            ))
            .await;
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
            match nodes {
                Ok(nodes) => {
                    trace!("Found {} Nodes.", nodes.len());
                    self.providers
                        .update_components_list(ComponentType::Node, nodes)
                        .await;
                }
                Err(err) => {
                    error!("Cannot get list node: {}", err);
                    res = Err(err);
                }
            }
            match gateways {
                Ok(gateways) => {
                    trace!("Found {} Gateways.", gateways.len());
                    self.providers
                        .update_components_list(ComponentType::Gateway, gateways)
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
    pub async fn reload_provider_map(&self) {
        let map = self.provider_service.get_provider_maps().await;
        if map.len() > 0 {
            trace!("Waiting for setting map {} worker providers", &map.len());
            self.workers.set_map_worker_provider(map).await;
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::persistence::services::WorkerService;
    use crate::{URL_GATEWAYS_LIST, URL_NODES_LIST};
    use anyhow::anyhow;
    use common::task_spawn;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use log::info;
    use std::env;
    use std::time::Instant;
    use test_util::helper::{load_env, mock_db_connection};

    const TEST_TIMEOUT: u64 = 15;

    fn run_mock_portal_server(body_node: &str, body_gateway: &str) -> MockServer {
        // Start a lightweight mock server.
        let server = MockServer::start();

        // Create a mock on the server.
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/mbr/node/list/verify");
            then.status(200)
                .header("content-type", "text/html; charset=UTF-8")
                .body(body_node);
        });
        // Create a mock on the server.
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/mbr/gateway/list/verify");
            then.status(200)
                .header("content-type", "text/html; charset=UTF-8")
                .body(body_gateway);
        });
        server
    }

    #[tokio::test]
    async fn test_scanner_run() -> Result<(), Error> {
        load_env();
        //init_logging();

        let body_node = r###"
        [{"id":"a7a5b6f2-1f2f-454a-baa7-17c3b3a5556e","userId":"b363ddf4-42cf-4ccf-89c2-8c42c531ac99","name":"node-net-highcpu2-europe-central2-a-03","appKey":"JYePW5CqdWTJRY2o-Yarsg","blockchain":"eth","ip":"34.116.249.195","countryCode":"PL","network":"mainnet","zone":"EU","status":"staked"},{"id":"0dc806f2-59b0-4300-b3e5-1e18b3095e10","userId":"b363ddf4-42cf-4ccf-89c2-8c42c531ac99","name":"node-net-highcpu2-us-central1-a-02","appKey":"ZCY7yfAnkt1R7gL_x9kCKw","blockchain":"eth","ip":"34.69.64.125","countryCode":"US","network":"mainnet","zone":"NA","status":"staked"},{"id":"22643ec7-f104-455b-a09c-d98ca91c9939","userId":"b363ddf4-42cf-4ccf-89c2-8c42c531ac99","name":"node-net-highcpu2-northamerica-northeast2-a-01","appKey":"Z160PQek6mArXXGHcM7gVQ","blockchain":"eth","ip":"34.130.199.16","countryCode":"CA","network":"mainnet","zone":"NA","status":"staked"},{"id":"b59259b2-4265-4baa-b650-2573429552c9","userId":"b363ddf4-42cf-4ccf-89c2-8c42c531ac99","name":"node-net-highcpu2-australia-southeast1-a-05","appKey":"QhoPF7lO8e9SyjeABRXEXA","blockchain":"eth","ip":"34.116.109.197","countryCode":"AU","network":"mainnet","zone":"OC","status":"staked"},{"id":"058a6e94-8b65-46ad-ab52-240a7cb2c36a","userId":"b363ddf4-42cf-4ccf-89c2-8c42c531ac99","name":"node-net-highcpu2-asia-southeast2-a-04","appKey":"lSP1lFN9I_izEzRi_jBapA","blockchain":"eth","ip":"34.101.146.31","countryCode":"ID","network":"mainnet","zone":"AS","status":"staked"},{"id":"a8811495-8cd6-40d9-870c-337e2306c84e","userId":"b363ddf4-42cf-4ccf-89c2-8c42c531ac99","name":"node-net-dot-highcpu2-australia-southeast1-a-05","appKey":"_Ny33sRzhMFxsloYsxblOg","blockchain":"dot","ip":"35.244.82.103","countryCode":"AU","network":"mainnet","zone":"OC","status":"staked"},{"id":"20471522-6bd8-4cd6-8230-a670316fc15b","userId":"b363ddf4-42cf-4ccf-89c2-8c42c531ac99","name":"node-net-dot-highcpu2-northamerica-northeast2-a-01","appKey":"vfb5izyLZ6DTPpvIdDuzyQ","blockchain":"dot","ip":"34.130.56.18","countryCode":"CA","network":"mainnet","zone":"NA","status":"staked"},{"id":"0ef09454-a282-4c6c-9ac4-a3e40ed44618","userId":"b363ddf4-42cf-4ccf-89c2-8c42c531ac99","name":"node-net-dot-highcpu2-europe-central2-a-03","appKey":"3JbN_KynklAMvHUjOrys-Q","blockchain":"dot","ip":"34.118.107.55","countryCode":"PL","network":"mainnet","zone":"EU","status":"staked"},{"id":"a77a2239-4860-4ce2-88ac-9e00a65b40d5","userId":"b363ddf4-42cf-4ccf-89c2-8c42c531ac99","name":"node-net-dot-highcpu2-us-central1-a-02","appKey":"25mAhlpTpJJblr5e1lQwpw","blockchain":"dot","ip":"35.239.87.20","countryCode":"US","network":"mainnet","zone":"NA","status":"staked"}]
        "###;
        let body_gateway = r###"
        [{"id":"614fca0e-8790-4c74-8b2c-1b071d5e6610","userId":"b363ddf4-42cf-4ccf-89c2-8c42c531ac99","appKey":"pzCF6ffJ26_w1YWW0yRoqw","name":"gw-net-highcpu2-us-central1-a-02-1","blockchain":"eth","ip":"35.224.36.173","countryCode":"US","network":"mainnet","zone":"NA","status":"staked"},{"id":"f17e3db1-07d9-4691-8769-5964ce2c771b","userId":"b363ddf4-42cf-4ccf-89c2-8c42c531ac99","appKey":"W9rPhv4fAYcSeIvNPwukPg","name":"gw-net-highcpu2-northamerica-northeast2-a-01-1","blockchain":"eth","ip":"34.130.55.254","countryCode":"CA","network":"mainnet","zone":"NA","status":"staked"},{"id":"c9282273-889a-4f52-8c13-d847cc70b283","userId":"b363ddf4-42cf-4ccf-89c2-8c42c531ac99","appKey":"d6UvCgcUt-rZYe8nQF4zNQ","name":"gw-net-highcpu2-europe-central2-a-03-1","blockchain":"eth","ip":"34.116.211.219","countryCode":"PL","network":"mainnet","zone":"EU","status":"staked"},{"id":"7bc87cee-1425-4bfd-8f24-ba8f25ee5e2e","userId":"b363ddf4-42cf-4ccf-89c2-8c42c531ac99","appKey":"cUH7LakFjSfLhrqoISFmNA","name":"gw-net-highcpu2-asia-southeast2-a-04-1","blockchain":"eth","ip":"34.101.135.131","countryCode":"ID","network":"mainnet","zone":"AS","status":"staked"}]
        "###;

        let nodes: Vec<ComponentInfo> = serde_json::from_str(body_node)?;
        let gateways: Vec<ComponentInfo> = serde_json::from_str(body_gateway)?;

        let portal = run_mock_portal_server(body_node, body_gateway);
        env::set_var("URL_PORTAL", portal.base_url());
        env::set_var("PATH_NODES_LIST", "mbr/node/list/verify");
        env::set_var("PATH_GATEWAYS_LIST", "mbr/gateway/list/verify");

        let arc_conn = Arc::new(mock_db_connection());
        //Get worker infos
        let worker_service = Arc::new(WorkerService::new(arc_conn.clone()));
        let provider_service = Arc::new(ProviderService::new(arc_conn.clone()));
        let all_workers = worker_service.clone().get_active().await;
        let provider_storage = Arc::new(ProviderStorage::default());

        log::debug!("Init with {:?} workers", all_workers.len());
        let worker_infos = Arc::new(WorkerInfoStorage::new(all_workers));

        //let (tx, mut rx) = mpsc::channel(1024);
        //Scanner for update provider list from portal
        let provider_scanner = ProviderScanner::new(
            URL_NODES_LIST.to_string(),
            URL_GATEWAYS_LIST.to_string(),
            provider_storage.clone(),
            worker_infos.clone(),
            provider_service.clone(),
        );
        task_spawn::spawn(async move { provider_scanner.run().await });

        let now = Instant::now();
        loop {
            if now.elapsed().as_secs() > TEST_TIMEOUT {
                return Err(anyhow!("Test Timeout"));
            }

            {
                let components = provider_storage.get_active_providers().await;
                if !components.is_empty() {
                    info!("Got {} provider", components.len());
                    assert_eq!(components.len(), nodes.len() + gateways.len());
                    for component in components.iter() {
                        assert!(
                            nodes.iter().any(|com| com.id == component.id)
                                || gateways.iter().any(|com| com.id == component.id)
                        );
                    }
                    break;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }

        Ok(())
    }
}
