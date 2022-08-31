use std::collections::hash_map::Entry;
use std::fmt::Formatter;
// Massbit chain
use codec::Decode;

use crate::SIGNER_PHRASE;
use anyhow::anyhow;
use common::component::ComponentType;
use common::job_manage::JobDetail::HttpRequest;
use common::job_manage::JobRole;
use common::jobs::Job;
use common::{BlockChainType, ComponentId, JobId, NetworkType, PlanId};
use log::info;
use serde::{Deserialize, Serialize};
use sp_core::crypto::Pair as _;
use sp_core::sr25519::Pair;
use sp_core::Bytes;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{
    compose_extrinsic, AccountId, AccountInfo, Api, ApiResult, PlainTipExtrinsicParams,
    UncheckedExtrinsicV4, XtStatus,
};
use tokio::sync::RwLock;
use tokio::time::sleep;

pub const MVP_EXTRINSIC_DAPI: &str = "Dapi";
const MVP_EXTRINSIC_SUBMIT_PROJECT_USAGE: &str = "submit_project_usage";
const MVP_EVENT_PROJECT_REGISTERED: &str = "ProjectRegistered";

pub const MVP_EXTRINSIC_FISHERMAN: &str = "Fisherman";
const MVP_EXTRINSIC_CREATE_JOB: &str = "create_job";
const MVP_EVENT_JOB_SUBMITTED: &str = "ProjectRegistered";
//const MVP_EVENT_CREATE_JOB: &str = "ProjectRegistered";

type ProjectIdString = String;
type Quota = String;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Projects(pub HashMap<ProjectIdString, Project>);

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    blockchain: String,
    network: String,
    quota: Quota,
    pub status: String,
}

#[derive(Default)]
pub struct ChainAdapter {
    pub ws_rpc_client: Option<WsRpcClient>,
    pub api: Option<Api<Pair, WsRpcClient, PlainTipExtrinsicParams>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct SubmitData {
    consumer_id: String,
    requests_count: isize,
    from_block_number: isize,
    to_block_number: isize,
}

type Data = Vec<u8>;

#[derive(Clone, Debug)]
pub struct SubmitJob {
    pub plan_id: Data,
    pub job_id: Data,
    pub job_name: Data,
    //For http request use task_name in task config ex: RoundTripTime/LatestBlock
    pub provider_id: Data,
    pub provider_type: Data,
    //node/gateway
    pub phase: Data,
    // regular
    pub url: Data,
    pub body: Data,
    pub method: u8,
    //post/get
    pub chain: Data,
    pub network: Data,
    pub response_type: Data,
    // json/string
    pub response_values: Data,
}

impl Default for SubmitJob {
    fn default() -> Self {
        let default: Data = vec![0; 36];
        SubmitJob {
            plan_id: default.clone(),
            job_id: default.clone(),
            job_name: default.clone(),
            provider_id: default.clone(),
            provider_type: default.clone(),
            phase: default.clone(),
            url: default.clone(),
            body: default.clone(),
            method: 0,
            chain: default.clone(),
            network: default.clone(),
            response_type: default.clone(),
            response_values: default.clone(),
        }
    }
}

#[allow(dead_code)]
#[derive(Decode, Debug)]
pub struct SubmitJobEventArgs {
    pub submitter: AccountId,
    pub job_id: Bytes,
}

#[allow(dead_code)]
#[derive(Decode, Debug)]
pub struct JobResultEventArgs {
    pub plan_id: Bytes,
    pub job_id: Bytes,
    pub job_name: Bytes, //For http request use task_name in task config ex: RoundTripTime/LatestBlock
    pub worker_id: Bytes,
    pub provider_id: Bytes,
    pub provider_type: Bytes, //node/gateway
    pub phase: Bytes,         // regular
    pub result_detail: Bytes,
    pub receive_timestamp: u64, //time the worker received result
    pub chain: Bytes,
    pub network: Bytes,
}

impl From<&Job> for SubmitJob {
    fn from(job: &Job) -> Self {
        if let HttpRequest(job_detail) = job.job_detail.clone() {
            let chain_info = job_detail.chain_info.unwrap_or_default();
            SubmitJob {
                plan_id: job.plan_id.as_bytes().try_into().unwrap(),
                job_id: job.job_id.as_bytes().try_into().unwrap(),
                job_name: job.job_name.as_bytes().try_into().unwrap(),
                provider_id: job.component_id.as_bytes().try_into().unwrap(),
                provider_type: job
                    .component_type
                    .to_string()
                    .as_bytes()
                    .try_into()
                    .unwrap(),
                phase: job.phase.to_string().as_bytes().try_into().unwrap(),
                url: job_detail.url.as_bytes().try_into().unwrap(),
                body: job_detail
                    .body
                    .unwrap_or_default()
                    .to_string()
                    .as_bytes()
                    .try_into()
                    .unwrap(),
                method: {
                    match job_detail.method.to_lowercase().as_str() {
                        "get" => 0,
                        "post" => 1,
                        _ => 1,
                    }
                },
                chain: chain_info.chain.to_string().as_bytes().try_into().unwrap(),
                network: chain_info.network.as_bytes().try_into().unwrap(),
                response_type: job_detail.response_type.as_bytes().try_into().unwrap(),
                response_values: serde_json::to_string(&job_detail.response_values)
                    .unwrap()
                    .as_bytes()
                    .try_into()
                    .unwrap(),
            }
        } else {
            SubmitJob::default()
        }
    }
}

#[allow(dead_code)]
#[derive(Decode, Debug)]
struct ProjectRegisteredEventArgs {
    project_id: Bytes,
    account_id: AccountId,
    chain_id: Bytes,
    quota: u64,
}

impl ProjectRegisteredEventArgs {
    fn project_id_to_string(&self) -> String {
        String::from_utf8_lossy(&*self.project_id).to_string()
    }
    fn get_blockchain_and_network(&self) -> (String, String) {
        let chain_id = String::from_utf8_lossy(&*self.chain_id).to_string();
        let chain_id = chain_id.split(".").collect::<Vec<&str>>();
        (chain_id[0].to_string(), chain_id[1].to_string())
    }
}

impl ChainAdapter {
    pub fn new() -> Self {
        info!("signer phrase: {}", &*SIGNER_PHRASE);
        let (derive_signer, _) = Pair::from_string_with_seed(&*SIGNER_PHRASE, None).unwrap();
        info!("derive_signer address:{:?}", derive_signer.public());
        let node_ip = "wss://chain-beta.massbitroute.net";
        let node_port = "443";
        let url = format!("{}:{}", node_ip, node_port);
        println!("Interacting with node on {}\n", url);
        let client = WsRpcClient::new(&url);
        let api = Api::<_, _, PlainTipExtrinsicParams>::new(client.clone())
            .map(|api| api.set_signer(derive_signer))
            .ok();
        ChainAdapter {
            ws_rpc_client: Some(client),
            api,
        }
    }
    pub fn submit_job(&self, job: &Job) -> Result<(), anyhow::Error> {
        info!("Submit job: {:?}", job);
        // set the recipient
        let api = self.api.as_ref().unwrap().clone();
        let submit_job = SubmitJob::from(job);
        info!(
            "[+] Composed Extrinsic:{}, api:{}, submit_job:{:?}",
            MVP_EXTRINSIC_FISHERMAN, MVP_EXTRINSIC_CREATE_JOB, submit_job
        );
        // the names are given as strings
        #[allow(clippy::redundant_clone)]
        let xt: UncheckedExtrinsicV4<_, _> = compose_extrinsic!(
            api,
            MVP_EXTRINSIC_FISHERMAN,
            MVP_EXTRINSIC_CREATE_JOB,
            submit_job.plan_id,
            submit_job.job_id,
            submit_job.job_name,
            submit_job.provider_id,
            submit_job.provider_type,
            submit_job.phase,
            submit_job.chain,
            submit_job.network,
            submit_job.response_type,
            submit_job.response_values,
            submit_job.url,
            Compact(submit_job.method),
            submit_job.body
        );

        // info!(
        //     "[+] Composed Extrinsic:{}, api:{}, submit_job:{:?}",
        //     MVP_EXTRINSIC_FISHERMAN, MVP_EXTRINSIC_CREATE_JOB, submit_job
        // );

        // send and watch extrinsic until InBlock
        let tx_hash = self
            .api
            .as_ref()
            .unwrap()
            .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)?;
        info!("[+] Transaction got included. Hash: {:?}", tx_hash);
        Ok(())
    }
    pub fn submit_jobs(&self, jobs: &[Job]) -> Result<(), anyhow::Error> {
        for job in jobs.iter() {
            if let Err(e) = self.submit_job(job) {
                info!("submit_jobs error:{:?}", e);
            };
        }
        Ok(())
    }

    pub async fn subscribe_event_submitted_job(
        &self,
        projects: Arc<RwLock<Projects>>,
    ) -> Result<(), anyhow::Error> {
        let (events_in, events_out) = channel();
        let api = self
            .api
            .as_ref()
            .ok_or(anyhow::Error::msg("Error: api is none"))?;
        api.subscribe_events(events_in)?;
        let mut retry = 10;
        loop {
            let event: ApiResult<ProjectRegisteredEventArgs> = api.wait_for_event(
                MVP_EXTRINSIC_FISHERMAN,
                MVP_EVENT_PROJECT_REGISTERED,
                None,
                &events_out,
            );

            match event {
                Ok(event) => {
                    info!("Got event: {:?}", event);
                    {
                        let mut projects_lock = projects.write().await;
                        match projects_lock.0.entry(event.project_id_to_string()) {
                            Entry::Occupied(o) => {
                                let project = o.into_mut();
                                project.quota = event.quota.to_string();
                            }
                            Entry::Vacant(v) => {
                                let (blockchain, network) = event.get_blockchain_and_network();
                                v.insert(Project {
                                    blockchain,
                                    network,
                                    quota: event.quota.to_string(),
                                    status: "staked".to_string(),
                                });
                            }
                        };
                        info!("projects quota update by event: {:?}", projects_lock);
                    }
                }
                Err(err) => {
                    info!("wait_for_event error:{:?}", err);
                    sleep(Duration::from_millis(1000)).await;
                    retry -= 1;
                    if retry <= 0 {
                        return Err(anyhow::Error::msg(format!(
                            "wait_for_event error in {} times",
                            retry
                        )));
                    }
                    continue;
                }
            }
        }
        Ok(())
    }
    pub async fn subscribe_event_update_quota(
        &self,
        projects: Arc<RwLock<Projects>>,
    ) -> Result<(), anyhow::Error> {
        let (events_in, events_out) = channel();
        let api = self
            .api
            .as_ref()
            .ok_or(anyhow::Error::msg("Error: api is none"))?;
        api.subscribe_events(events_in)?;
        let mut retry = 10;
        loop {
            let event: ApiResult<ProjectRegisteredEventArgs> = api.wait_for_event(
                MVP_EXTRINSIC_DAPI,
                MVP_EVENT_PROJECT_REGISTERED,
                None,
                &events_out,
            );

            match event {
                Ok(event) => {
                    info!("Got event: {:?}", event);
                    {
                        let mut projects_lock = projects.write().await;
                        match projects_lock.0.entry(event.project_id_to_string()) {
                            Entry::Occupied(o) => {
                                let project = o.into_mut();
                                project.quota = event.quota.to_string();
                            }
                            Entry::Vacant(v) => {
                                let (blockchain, network) = event.get_blockchain_and_network();
                                v.insert(Project {
                                    blockchain,
                                    network,
                                    quota: event.quota.to_string(),
                                    status: "staked".to_string(),
                                });
                            }
                        };
                        info!("projects quota update by event: {:?}", projects_lock);
                    }
                }
                Err(err) => {
                    info!("wait_for_event error:{:?}", err);
                    sleep(Duration::from_millis(1000)).await;
                    retry -= 1;
                    if retry <= 0 {
                        return Err(anyhow::Error::msg(format!(
                            "wait_for_event error in {} times",
                            retry
                        )));
                    }
                    continue;
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for ChainAdapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MvpAdapter")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::{load_yaml, App};
    use common::logger::init_logger;

    use sp_keyring::AccountKeyring;
    use substrate_api_client::rpc::WsRpcClient;
    use substrate_api_client::Api;
    use substrate_api_client::{AccountInfo, PlainTipExtrinsicParams};
    use test_util::helper::{init_logging, load_env, mock_job, JobName};

    #[tokio::test]
    async fn test_call_chain() {
        load_env();
        init_logging();
        let mut job = mock_job(
            &JobName::RoundTripTime,
            "http://34.116.86.153/_rtt",
            "job_id1",
            &Default::default(),
        );
        // Submit job
        let adapter = ChainAdapter::new();
        adapter.submit_jobs(&vec![job]);

        // listen event
    }

    #[tokio::test]
    async fn test_call_chain_example() {
        //env_logger::init();
        init_logger("test_chain");
        let url = get_node_url_from_cli();

        let client = WsRpcClient::new(&url);
        let mut api = Api::<_, _, PlainTipExtrinsicParams>::new(client).unwrap();

        // get some plain storage value
        let result: u128 = api
            .get_storage_value("Balances", "TotalIssuance", None)
            .unwrap()
            .unwrap();
        println!("[+] TotalIssuance is {}", result);

        let proof = api
            .get_storage_value_proof("Balances", "TotalIssuance", None)
            .unwrap();
        println!("[+] StorageValueProof: {:?}", proof);

        // get StorageMap
        let account = AccountKeyring::Alice.public();
        let result: AccountInfo = api
            .get_storage_map("System", "Account", account, None)
            .unwrap()
            .or_else(|| Some(AccountInfo::default()))
            .unwrap();
        println!("[+] AccountInfo for Alice is {:?}", result);

        // get StorageMap key prefix
        let result = api.get_storage_map_key_prefix("System", "Account").unwrap();
        println!("[+] key prefix for System Account map is {:?}", result);

        // get Alice's AccountNonce with api.get_nonce()
        let signer = AccountKeyring::Alice.pair();
        api.signer = Some(signer);
        println!("[+] Alice's Account Nonce is {}", api.get_nonce().unwrap());
    }
    pub fn get_node_url_from_cli() -> String {
        let node_ip = "wss://chain.massbitroute.net";
        let node_port = "443";
        let url = format!("{}:{}", node_ip, node_port);
        println!("Interacting with node on {}\n", url);
        url
    }
}
