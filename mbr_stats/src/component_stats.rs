use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::chain_adapter::{ChainAdapter, Projects};

use jsonrpsee::core::client::{Subscription, SubscriptionClientT};

use jsonrpsee::rpc_params;
use jsonrpsee::ws_client::WsClientBuilder;
use log::info;
use prometheus_http_query::aggregations::sum;

use prometheus_http_query::{Aggregate, Client as PrometheusClient, InstantVector, Selector};
use regex::Regex;
use serde_json::Value;

use std::convert::TryInto;
use std::fmt::Formatter;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::Api;

use crate::{DEFAULT_HTTP_REQUEST_TIMEOUT_MS, PORTAL_AUTHORIZATION, SCHEME};
use anyhow::Error;
use reqwest::Client;
use sp_core::sr25519::Pair;
use sp_core::Pair as PairTrait;
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};
use tokio::sync::RwLock;
use tokio::task;
use tokio::time::{sleep, Duration};

type TimeStamp = i64;

const NUMBER_BLOCK_FOR_COUNTING: isize = 2;
const DATA_NAME: &str = "nginx_vts_filter_requests_total";
// const PROJECT_FILTER: &str = ".*::proj::api_method";
const PROJECT_FILTER: &str = ".*::project::.*";
const PROJECT_FILTER_PROJECT_ID_REGEX: &str =
    r"::project::([0-9a-fA-F]{8}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{12})";
const UPDATE_PROJECT_QUOTA_INTERVAL: u64 = 10; //sec

#[derive(EnumString, EnumIter, Eq, Hash, PartialEq, Debug, Clone)]
pub enum ChainId {
    #[strum(serialize = "snake_case")]
    EthMainnet,
    // #[strum(serialize = "snake_case")]
    DotMainnet,
}

impl ChainId {
    pub fn to_string_prometheus(&self) -> String {
        match self {
            ChainId::EthMainnet => "eth-mainnet".to_string(),
            ChainId::DotMainnet => "dot-mainnet".to_string(),
        }
    }
    pub fn from_string(chain: &String, network: &String) -> Result<Self, Error> {
        let chain_id = format!("{}_{}", chain, network);
        Ok(ChainId::from_str(chain_id.as_str())?)
    }
}

#[derive(Debug)]
pub enum ComponentType {
    Node,
    Dapi,
    Gateway,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ConfigData;

#[derive(Debug)]
pub struct ComponentStats {
    // input file
    pub config_data_uri: String,
    // config data
    pub config_data: Option<ConfigData>,
    // Url
    pub prometheus_gateway_url: String,
    pub prometheus_node_url: String,
    // Massbit verification protocol chain url
    pub mvp_url: String,
    pub signer_phrase: String,

    // For collecting data
    pub gateway_adapters: HashMap<ChainId, Arc<DataCollectionAdapter>>,
    pub node_adapters: HashMap<ChainId, Arc<DataCollectionAdapter>>,
    // Projects data
    pub list_project_url: String,
    pub projects: Arc<RwLock<Projects>>,

    // For submit data
    pub chain_adapter: Arc<ChainAdapter>,
}

#[derive(Clone)]
pub struct DataCollectionAdapter {
    client: PrometheusClient,
}

impl std::fmt::Debug for DataCollectionAdapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let url = self.client.base_url();
        write!(f, "DataCollectionAdapter<{}>", url)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct QueryData {
    name: String,
    tags: Vec<String>,
    value: isize,
    from_block_number: isize,
    to_block_number: isize,
    from_timestamp: i64,
    to_timestamp: i64,
    is_keep: bool,
}

pub struct StatsBuilder {
    inner: ComponentStats,
}

impl ComponentStats {
    pub async fn builder(path: &str, signer_phrase: &str) -> StatsBuilder {
        let url = if path.starts_with("wss") && !path.ends_with(":443") {
            format!("{}:443", path)
        } else {
            path.to_string()
        };
        info!("Massbit chain path: {}", url,);
        // RPSee client for subscribe new block
        let json_rpc_client = WsClientBuilder::default()
            .build(&url)
            .await
            .expect("Cannot build JsonRpcClient");
        info!(
            "Massbit chain path: {}, chain client: {:?}",
            url, json_rpc_client,
        );
        info!("signer_phrase:{:?}", signer_phrase);
        let (derive_signer, _) = Pair::from_string_with_seed(signer_phrase, None).unwrap();

        info!("derive_signer address:{:?}", derive_signer.public());

        let ws_rpc_client = WsRpcClient::new(&url);

        let api = Api::new(ws_rpc_client.clone())
            .map(|api| api.set_signer(derive_signer))
            .expect("Cannot set signer");
        let chain_adapter = ChainAdapter {
            json_rpc_client,
            ws_rpc_client,
            api,
        };
        let component_stats = ComponentStats {
            config_data_uri: "".to_string(),
            config_data: None,
            prometheus_gateway_url: "".to_string(),
            chain_adapter: Arc::new(chain_adapter),
            mvp_url: url,

            signer_phrase: "".to_string(),
            gateway_adapters: Default::default(),
            node_adapters: Default::default(),
            list_project_url: "".to_string(),
            prometheus_node_url: "".to_string(),
            projects: Arc::new(Default::default()),
        };
        StatsBuilder {
            inner: component_stats,
        }
    }

    pub async fn get_projects_quota_list(
        portal_adapter: &Client,
        projects: Arc<RwLock<Projects>>,
        list_project_url: &String,
        status: &str,
    ) -> Result<(), anyhow::Error> {
        let res = portal_adapter
            .get(list_project_url)
            .header("Authorization", PORTAL_AUTHORIZATION.as_str())
            .timeout(Duration::from_millis(DEFAULT_HTTP_REQUEST_TIMEOUT_MS))
            .send()
            .await?
            .text()
            .await?;
        // let res = reqwest::get(list_project_url).await?.text().await?;
        let mut projects_new: Projects = serde_json::from_str(res.as_str())?;
        // Filter by status
        projects_new
            .0
            .retain(|_id, project| project.status == *status);
        {
            let mut lock = projects.write().await;
            lock.0 = projects_new.0;
            info!("projects quota update by portal: {:?}", lock.0);
        }

        Ok(())
    }

    fn capture_project_id(filter: &str) -> Option<String> {
        let re = Regex::new(PROJECT_FILTER_PROJECT_ID_REGEX).unwrap();
        let project_id = re
            .captures(filter)
            .and_then(|id| id.get(1).and_then(|id| Some(id.as_str().to_string())));
        //info!("filter: {}, project_id:{:?}",filter, project_id);
        project_id
    }

    async fn get_request_number(
        &self,
        filter_name: &str,
        filter_value: &str,
        data_name: &str,
        time: TimeStamp,
        chain_id: &ChainId,
    ) -> anyhow::Result<HashMap<String, usize>> {
        let q: InstantVector = Selector::new()
            .metric(data_name)
            .regex_match(filter_name, filter_value)
            .with("code", "2xx")
            .try_into()?;
        let q = sum(q, Some(Aggregate::By(&["filter"])));
        let adapter = self.gateway_adapters.get(chain_id).unwrap();
        info!(
            "time: {}, data_name: {}, filter_name: {},filter_value:{}, adapter: {:?}",
            time, data_name, filter_name, filter_value, adapter
        );
        let res = adapter.client.query(q, Some(time), None).await?;

        let res = if let Some(instant) = res.as_instant() {
            Ok({
                let mut projects: HashMap<String, usize> = HashMap::new();
                for iv in instant.iter() {
                    if let Some(filter) = iv.metric().get("filter") {
                        let value = iv.sample().value() as usize;
                        if let Some(project_id) =
                            ComponentStats::capture_project_id(filter.as_str())
                        {
                            match projects.entry(project_id) {
                                Entry::Occupied(o) => {
                                    let tmp = o.into_mut();
                                    *tmp += value;
                                }
                                Entry::Vacant(v) => {
                                    v.insert(value);
                                }
                            };
                        };
                    };
                }
                projects
            })
        } else {
            Err(anyhow::Error::msg("Cannot parse response"))
        };
        //info!("Hashmap res: {:#?}", res);
        res
    }

    async fn subscribe_finalized_heads(&self) -> Result<Subscription<Value>, anyhow::Error> {
        let client = &self.chain_adapter.json_rpc_client;
        let subscribe_finalized_heads = client
            .subscribe::<Value>(
                "chain_subscribeFinalizedHeads",
                rpc_params![],
                "chain_unsubscribeFinalizedHeads",
            )
            .await;
        info!("subscribe_finalized_heads: {:?}", subscribe_finalized_heads);
        Ok(subscribe_finalized_heads?)
    }

    async fn loop_get_request_number(
        &self,
        mut subscribe: Subscription<Value>,
    ) -> Result<(), anyhow::Error> {
        // wait for new block
        let mut last_count_block: isize = -1;

        loop {
            for chain_id in ChainId::iter() {
                let res = subscribe.next().await;
                if let Some(Ok(res)) = res {
                    info!("received {:?}", res);
                    if let Some(block_number) = res.get("number") {
                        let block_number = isize::from_str_radix(
                            block_number.as_str().unwrap().trim_start_matches("0x"),
                            16,
                        )?;
                        info!("block_number {:?}", block_number);
                        if last_count_block == -1 {
                            last_count_block = block_number;
                            continue;
                        } else if block_number - last_count_block >= NUMBER_BLOCK_FOR_COUNTING {
                            // Set new count block
                            let current_count_block = last_count_block + NUMBER_BLOCK_FOR_COUNTING;
                            // Fixme: need decode block for getting timestamp. For now use system time.
                            let current_count_block_timestamp =
                                (SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
                                    as TimeStamp;

                            // Get request number
                            let projects_request = self
                                .get_request_number(
                                    "filter",
                                    PROJECT_FILTER,
                                    DATA_NAME,
                                    current_count_block_timestamp,
                                    &chain_id,
                                )
                                .await;

                            match projects_request {
                                Ok(projects_request) => {
                                    info!("projects_request: {:?}", projects_request);
                                    let project_quota = self.projects.clone();
                                    if let Err(e) = self
                                        .chain_adapter
                                        .submit_projects_usage(project_quota, projects_request)
                                        .await
                                    {
                                        info!("Submit projects usage error: {:?}", e)
                                    }
                                }

                                Err(e) => info!("get_request_number error: {}", e),
                            }
                            last_count_block = current_count_block;
                        }
                    }
                }
            }
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        // info!("test_sign_extrinsic...");
        // self.chain_adapter.test_sign_extrinsic();

        let projects = self.projects.clone();
        let chain_adapter = self.chain_adapter.clone();
        let list_project_url = self.list_project_url.clone();

        // Update quota list
        task::spawn(async move {
            let client_builder = reqwest::ClientBuilder::new();
            let portal_adapter = client_builder
                .danger_accept_invalid_certs(true)
                .build()
                .expect("Cannot build portal_adapter");
            loop {
                let res = Self::get_projects_quota_list(
                    &portal_adapter,
                    projects.clone(),
                    &list_project_url,
                    "staked",
                )
                .await;
                if res.is_err() {
                    info!("Get projects quota error: {:?}", res);
                }
                sleep(Duration::from_secs(UPDATE_PROJECT_QUOTA_INTERVAL)).await;
            }
        });
        let projects = self.projects.clone();
        // Update quota list
        task::spawn(async move {
            info!("Subscribe_event");
            loop {
                let res = chain_adapter
                    .subscribe_event_update_quota(projects.clone())
                    .await;

                info!("Re-subscribe event, res: {:?}", res);
            }
        });

        // subscribe finalized header
        let subscribe_finalized_heads = self.subscribe_finalized_heads().await?;

        // Get request number over time to submit to chain
        self.loop_get_request_number(subscribe_finalized_heads)
            .await?;

        Ok(())
    }
}
impl StatsBuilder {
    async fn get_prometheus_client(url: &String) -> anyhow::Result<PrometheusClient> {
        let client = PrometheusClient::try_from(url.as_str())
            .map_err(|err| anyhow::Error::msg(format!("{:?}", err)));
        client
    }

    pub async fn with_prometheus_url(mut self, path: &str) -> StatsBuilder {
        //https://node-eth-mainnet.stat.mbr.massbitroute.net/__internal_prometheus
        // self.inner.prometheus_gateway_url = format!("https://gateway-[[chain_id]].{}", path);
        // self.inner.prometheus_node_url = format!("https://node-[[chain_id]].{}", path);
        // Create adapters for prometheus
        for component_type in vec!["node", "gw"] {
            let adapters = match component_type {
                "node" => &mut self.inner.node_adapters,
                _ => &mut self.inner.gateway_adapters,
            };

            for chain_id in ChainId::iter() {
                let url = if component_type == "node" {
                    format!(
                        "{}://node-{}.{}",
                        SCHEME.to_http_string(),
                        chain_id.to_string_prometheus(),
                        path,
                    )
                } else {
                    format!(
                        "{}://gateway-{}.{}",
                        SCHEME.to_http_string(),
                        chain_id.to_string_prometheus(),
                        path,
                    )
                };

                info!("Prometheus url: {}", url);

                adapters.insert(
                    chain_id.clone(),
                    Arc::new(DataCollectionAdapter {
                        //data: Default::default(),
                        client: Self::get_prometheus_client(&url)
                            .await
                            .expect(&format!("Cannot get prometheus_client by url: {}", url)),
                    }),
                );
            }
        }

        self
    }

    pub async fn with_mvp_url(mut self, path: String) -> StatsBuilder {
        let url = if path.starts_with("wss") {
            format!("{}:443", path)
        } else {
            path.to_string()
        };
        self.inner.mvp_url = url;
        // RPSee client for subscribe new block
        let json_rpc_client = WsClientBuilder::default()
            .build(&path)
            .await
            .expect("Cannot build JsonRpcClient");
        info!(
            "Massbit chain path: {}, chain client: {:?}",
            path, json_rpc_client,
        );

        let (derive_signer, _) =
            Pair::from_string_with_seed(self.inner.signer_phrase.as_str(), None).unwrap();

        info!("derive_signer address:{:?}", derive_signer.public());

        let ws_rpc_client = WsRpcClient::new(&self.inner.mvp_url);

        let api = Api::new(ws_rpc_client.clone())
            .map(|api| api.set_signer(derive_signer))
            .expect("Cannot set signer");
        let chain_adapter = ChainAdapter {
            json_rpc_client,
            ws_rpc_client,
            api,
        };
        self.inner.chain_adapter = Arc::new(chain_adapter);
        self
    }
    pub fn with_list_project_url(mut self, path: String) -> StatsBuilder {
        self.inner.list_project_url = path;
        self
    }
    pub fn with_signer_phrase(mut self, signer_phrase: String) -> Self {
        self.inner.signer_phrase = signer_phrase;
        self
    }
    pub fn build(self) -> ComponentStats {
        self.inner
    }
}
