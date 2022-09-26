use std::fmt::Formatter;
// Massbit chain
use codec::Decode;

use crate::{REPORT_CALLBACK, SCHEDULER_AUTHORIZATION, SIGNER_PHRASE, URL_CHAIN};
use anyhow::anyhow;
use common::component::{ChainInfo, ComponentType};
use common::job_manage::JobDetail::HttpRequest;
use common::job_manage::{JobDetail, JobResultDetail, JobRole};
use common::jobs::{Job, JobResult};
use common::tasks::http_request::{
    HttpResponseValues, JobHttpRequest, JobHttpResponse, JobHttpResponseDetail, JobHttpResult,
};
use common::util::get_current_time;
use common::{BlockChainType, JobId, Timestamp, COMMON_CONFIG};
use hex::FromHex;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use sp_core::crypto::Pair as _;
use sp_core::sr25519::Pair;
use sp_core::Bytes as SpCoreBytes;
use std::collections::HashMap;
use std::fmt::Debug;
use std::num::ParseIntError;
use std::str::{from_utf8, FromStr};

use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use std::thread;
use std::time::Duration;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{
    compose_extrinsic, AccountId, Api, ApiResult, EventsDecoder, PlainTipExtrinsicParams, Raw,
    UncheckedExtrinsicV4, XtStatus,
};

use tokio::time::sleep;

//static SUBMIT_JOB_RUNNING: AtomicBool = AtomicBool::new(false);

pub const MVP_EXTRINSIC_DAPI: &str = "Dapi";

pub const MVP_EXTRINSIC_FISHERMAN: &str = "Fisherman";
const MVP_EXTRINSIC_CREATE_JOB: &str = "create_job";
const MVP_EVENT_JOB_RESULT: &str = "NewJobResult";
const SEND_JOB_INTERVAL: u64 = 1000; //ms

type ProjectIdString = String;
type Quota = String;
type Data = Vec<u8>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultLatestBlock {
    hash: Vec<u8>,
    number: Vec<u8>,
}

#[derive(Clone, PartialEq, Eq, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ReturnJob {
    plan_id: Data,
    job_name: Data,
    job_id: Data,
    provider_id: Data,
    provider_type: Data,
    phase: Data,
    chain: Data,
    network: Data,
    response_type: Data,
    response_values: Data,
    url: Data,
    method: ApiMethod,
    headers: Vec<(Data, Data)>,
    payload: Data,
}

impl ReturnJob {
    fn to_job(&self, job_id: JobId) -> Job {
        let chain_info = Some(ChainInfo {
            chain: BlockChainType::from_str(from_utf8(&self.chain).unwrap_or_default())
                .unwrap_or_default(),
            network: from_utf8(&self.network).unwrap().to_string(),
        });

        let job_detail = JobHttpRequest {
            url: from_utf8(&self.url).unwrap_or_default().to_string(),
            chain_info,
            method: self.method.to_string(),
            headers: Default::default(),
            body: None,
            response_type: from_utf8(&self.response_type)
                .unwrap_or_default()
                .to_string(),
            response_values: serde_json::from_str(
                from_utf8(&self.response_values).unwrap_or_default(),
            )
            .unwrap_or_default(),
        };
        Job {
            job_id,
            job_type: "HttpRequest".to_string(),
            job_name: from_utf8(&self.job_name).unwrap_or_default().to_string(),
            plan_id: from_utf8(&self.plan_id).unwrap_or_default().to_string(),
            component_id: from_utf8(&self.provider_id).unwrap_or_default().to_string(),
            component_type: ComponentType::from_str(
                from_utf8(&self.provider_type).unwrap_or_default(),
            )
            .unwrap_or_default(),
            job_detail: JobDetail::HttpRequest(job_detail),
            phase: JobRole::from_str(from_utf8(&self.phase).unwrap_or_default())
                .unwrap_or_default(),
            priority: 0,
            expected_runtime: 0,
            parallelable: false,
            timeout: 0,
            component_url: "".to_string(),
            repeat_number: 0,
            interval: 0,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Decode)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize, Debug))]
pub enum ApiMethod {
    Get,
    Post,
}

impl ToString for ApiMethod {
    fn to_string(&self) -> String {
        match self {
            ApiMethod::Get => "get".to_string(),
            ApiMethod::Post => "post".to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Default, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ReturnJobResult {
    result: Data,
    timestamp: u128,
    is_success: bool,
}

#[derive(Clone, PartialEq, Eq, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct NewJobResult {
    job: ReturnJob,
    job_result: ReturnJobResult,
}

#[derive(Clone, PartialEq, Eq, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct NewJobResults {
    results: Vec<(ReturnJob, ReturnJobResult)>,
}

impl From<(ReturnJob, ReturnJobResult)> for NewJobResult {
    fn from(data: (ReturnJob, ReturnJobResult)) -> Self {
        (job, job_result) = data;
        NewJobResult { job, job_result }
    }
}
impl Into<JobResult> for NewJobResult {
    fn into(self) -> JobResult {
        let return_job_result = self.job_result.clone();
        let result = from_utf8(&*return_job_result.result).unwrap();
        let job_id: JobId = from_utf8(&self.job.job_id).unwrap().to_string();
        let chain_info = Some(ChainInfo {
            chain: BlockChainType::from_str(from_utf8(&self.job.chain).unwrap())
                .unwrap_or_default(),
            network: from_utf8(&self.job.network).unwrap().to_string(),
        });
        let job = self.job.to_job(job_id.clone());
        let is_success = return_job_result.is_success;

        let http_response = match job.job_name.as_str() {
            "RoundTripTime" => {
                if is_success {
                    let body = result.trim_matches('\n');
                    let response_duration = Timestamp::from_str(body).unwrap() / 1000i64;
                    JobHttpResponse {
                        request_timestamp: return_job_result.timestamp as i64,
                        response_duration,
                        detail: JobHttpResponseDetail::Body(body.to_string()),
                        http_code: 200,
                        error_code: 0,
                        message: "".to_string(),
                    }
                } else {
                    JobHttpResponse {
                        request_timestamp: return_job_result.timestamp as i64,
                        response_duration: 0,
                        detail: JobHttpResponseDetail::Body("".to_string()),
                        http_code: 0,
                        error_code: 1,
                        message: "not success".to_string(),
                    }
                }
            }
            _ => {
                if is_success {
                    let response_duration = Timestamp::from_str(result.trim_matches('\n'));
                    match response_duration {
                        Ok(response_duration) => JobHttpResponse {
                            request_timestamp: return_job_result.timestamp as i64,
                            response_duration,
                            detail: JobHttpResponseDetail::Values(HttpResponseValues::new(
                                serde_json::from_str(result).unwrap(),
                            )),
                            http_code: 200,
                            error_code: 0,
                            message: "".to_string(),
                        },
                        Err(err) => JobHttpResponse {
                            request_timestamp: return_job_result.timestamp as i64,
                            response_duration: 0,
                            detail: JobHttpResponseDetail::Values(HttpResponseValues::new(
                                HashMap::new(),
                            )),
                            http_code: 0,
                            error_code: 1,
                            message: format!("cannot parse result {}, error: {}", result, err),
                        },
                    }
                } else {
                    JobHttpResponse {
                        request_timestamp: return_job_result.timestamp as i64,
                        response_duration: 0,
                        detail: JobHttpResponseDetail::Values(HttpResponseValues::new(
                            HashMap::new(),
                        )),
                        http_code: 0,
                        error_code: 1,
                        message: "not success".to_string(),
                    }
                }
            }
        };
        let job_http_result = JobHttpResult::new(job.clone(), http_response);
        JobResult {
            plan_id: job.plan_id,
            job_id,
            job_name: job.job_name,
            worker_id: "onchain-worker".to_string(),
            provider_id: job.component_id,
            provider_type: job.component_type,
            phase: job.phase,
            result_detail: JobResultDetail::HttpRequest(job_http_result),
            receive_timestamp: get_current_time(),
            chain_info,
        }
    }
}

impl NewJobResult {
    pub async fn send_results(
        result_callback: &str,
        results: Vec<NewJobResult>,
    ) -> Result<(), anyhow::Error> {
        // Convert to NewJobResult
        let mut results: Vec<JobResult> = results.into_iter().map(|result| result.into()).collect();
        // send results
        // Edit for gran only
        let mut filtered_results = HashMap::new();
        for result in results {
            let values = filtered_results
                .entry(result.job_id.clone())
                .or_insert_with(|| result);
        }
        let results: Vec<JobResult> = filtered_results.values().cloned().collect();
        // End Edit for gran only
        let call_back = result_callback.to_string();
        info!("Send {} results to: {}", results.len(), call_back);
        debug!("Results: {:?}", results);
        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        let body = serde_json::to_string(&results)?;
        trace!("Body content: {}", body);
        info!("sending body len: {}", body.len());
        let result = client
            .post(call_back)
            .header("content-type", "application/json")
            .header("authorization", &*SCHEDULER_AUTHORIZATION)
            .body(body)
            .timeout(Duration::from_millis(
                COMMON_CONFIG.default_http_request_timeout_ms,
            ))
            .send()
            .await;
        info!("Send response: {:?}", result);
        match result {
            Ok(_res) => Ok(()),
            Err(err) => Err(anyhow!(format!("{:?}", &err))),
        }
    }
}

#[allow(dead_code)]
#[derive(Decode, Debug)]
pub struct JobResultEventArgs {
    pub submitter: AccountId,
    pub job_id: SpCoreBytes,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Projects(pub HashMap<ProjectIdString, Project>);

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    blockchain: String,
    network: String,
    quota: Quota,
    pub status: String,
}

#[derive(Clone)]
pub struct ChainAdapter {
    pub ws_rpc_client: WsRpcClient,
    pub api: Api<Pair, WsRpcClient, PlainTipExtrinsicParams>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct SubmitData {
    consumer_id: String,
    requests_count: isize,
    from_block_number: isize,
    to_block_number: isize,
}

#[derive(Clone, Debug, Default)]
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
    pub headers: Vec<(Data, Data)>,
}

#[allow(dead_code)]
#[derive(Decode, Debug)]
pub struct SubmitJobEventArgs {
    pub submitter: AccountId,
    pub job_id: SpCoreBytes,
}

impl From<&Job> for SubmitJob {
    fn from(job: &Job) -> Self {
        if let HttpRequest(job_detail) = job.job_detail.clone() {
            let chain_info = job_detail.chain_info.unwrap_or_default();
            let x_api_key: Data = job_detail
                .headers
                .get("x-api-key")
                .unwrap_or(&"".to_string())
                .as_bytes()
                .to_vec();
            let host = job_detail
                .headers
                .get("host")
                .unwrap_or(&"".to_string())
                .as_bytes()
                .to_vec();
            let content_type = job_detail
                .headers
                .get("content-type")
                .unwrap_or(&"".to_string())
                .as_bytes()
                .to_vec();
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
                url: job_detail
                    .url
                    .replace("https", "http")
                    .as_bytes()
                    .try_into()
                    .unwrap(),
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
                headers: vec![
                    ("x-api-key".as_bytes().try_into().unwrap(), x_api_key),
                    ("host".as_bytes().try_into().unwrap(), host),
                    ("content-type".as_bytes().try_into().unwrap(), content_type),
                ],
            }
        } else {
            SubmitJob::default()
        }
    }
}

#[allow(dead_code)]
#[derive(Decode, Debug)]
struct ProjectRegisteredEventArgs {
    project_id: SpCoreBytes,
    account_id: AccountId,
    chain_id: SpCoreBytes,
    quota: u64,
}

impl ChainAdapter {
    pub fn new() -> Self {
        info!("signer phrase: {}", &*SIGNER_PHRASE);
        let (derive_signer, _) = Pair::from_string_with_seed(&*SIGNER_PHRASE, None).unwrap();
        info!("derive_signer address:{:?}", derive_signer.public());
        let node_ip = &*URL_CHAIN;
        let url = if node_ip.starts_with("wss") {
            format!("{}:443", node_ip)
        } else {
            node_ip.to_string()
        };

        info!("Interacting with node on {}\n", url);
        let client = WsRpcClient::new(&url);
        let api = Api::<_, _, PlainTipExtrinsicParams>::new(client.clone())
            .map(|api| api.set_signer(derive_signer))
            .expect("Cannot create api");
        ChainAdapter {
            ws_rpc_client: client,
            api,
        }
    }
    pub async fn submit_job(&self, job: &Job) -> Result<(), anyhow::Error> {
        // while SUBMIT_JOB_RUNNING.load(Ordering::Relaxed) {
        //     sleep(Duration::from_millis(SEND_JOB_INTERVAL)).await;
        // }
        // SUBMIT_JOB_RUNNING.store(true, Ordering::Relaxed);
        info!("Submit job: {:?}", job);
        // set the recipient
        info!("nonce before: {:?}", &self.api.get_nonce());
        let submit_job = SubmitJob::from(job);
        info!(
            "[+] Composed Extrinsic:{}, api:{}, submit_job:{:?}",
            MVP_EXTRINSIC_FISHERMAN, MVP_EXTRINSIC_CREATE_JOB, submit_job
        );
        info!(
            "[+] submit_job.body: {}, submit_job.response_values {},",
            from_utf8(&submit_job.body).unwrap(),
            from_utf8(&submit_job.response_values).unwrap()
        );
        // the names are given as strings
        #[allow(clippy::redundant_clone)]
        let xt: UncheckedExtrinsicV4<_, _> = compose_extrinsic!(
            &self.api,
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
            submit_job.method,
            submit_job.headers,
            submit_job.body
        );

        // send and watch extrinsic until InBlock
        let tx_hash = self.api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock);
        info!("nonce after: {:?}", &self.api.get_nonce());
        sleep(Duration::from_millis(SEND_JOB_INTERVAL)).await;
        // SUBMIT_JOB_RUNNING.store(false, Ordering::Relaxed);

        info!("[+] Transaction got included. Hash: {:?}", tx_hash?);
        Ok(())
    }
    pub async fn submit_jobs(&self, jobs: &[Job]) -> Result<(), anyhow::Error> {
        if jobs.is_empty() {
            return Ok(());
        }
        for job in jobs.iter() {
            if let Err(e) = self.submit_job(job).await {
                info!("submit_jobs error:{:?}", e);
            };
        }
        Ok(())
    }

    pub async fn subscribe_event_onchain_worker(&self) {
        let (events_in, events_out) = channel();

        let api = Arc::new(self.api.clone());
        let clone_api = api.clone();
        //api.subscribe_events(events_in)?;
        let _eventsubscriber = thread::Builder::new()
            .name("eventsubscriber".to_owned())
            .spawn(move || {
                clone_api.subscribe_events(events_in).unwrap();
            })
            .unwrap();
        warn!("[+] Subscribed to events. waiting...");

        Self::wait_for_event(
            &api,
            MVP_EXTRINSIC_FISHERMAN,
            MVP_EVENT_JOB_RESULT,
            None,
            events_out,
        )
        .await;
    }

    pub async fn wait_for_event(
        api: &Api<Pair, WsRpcClient, PlainTipExtrinsicParams>,
        module: &str,
        variant: &str,
        decoder: Option<EventsDecoder>,
        receiver: Receiver<String>,
    ) {
        let event_decoder = match decoder {
            Some(d) => d,
            None => EventsDecoder::new(api.metadata.clone()),
        };

        loop {
            let event_str = receiver.recv();
            if event_str.is_err() {
                error!("recv error: {:?}", event_str);
                sleep(Duration::from_millis(2000)).await;
                continue;
            }
            let event_str = event_str
                .unwrap()
                .trim_matches('\"')
                .trim_start_matches("0x")
                .to_string();
            let event_str = Vec::from_hex(event_str);
            if event_str.is_err() {
                error!("from_hex error: {:?}", event_str);
                continue;
            }
            let _events = event_decoder.decode_events(&mut event_str.unwrap().as_slice());
            info!("wait for raw event");
            let mut vec_job_results = Vec::new();
            match _events {
                Ok(raw_events) => {
                    for (phase, event) in raw_events.into_iter() {
                        info!("Decoded Event: {:?}, {:?}", phase, event);
                        match event {
                            Raw::Event(raw) if raw.pallet == module && raw.variant == variant => {
                                let job_result: ApiResult<NewJobResults> =
                                    NewJobResults::decode(&mut &raw.data[..]).map_err(|e| e.into());
                                warn!("job_result: {:?}", job_result);
                                // Convert to JobResult
                                match job_result {
                                    Ok(job_results) => {
                                        let vec_res: Vec<NewJobResult> = job_results
                                            .results
                                            .into_iter()
                                            .map(|res| NewJobResult::from(res))
                                            .collect();
                                        vec_job_results.extend(vec_res);
                                        // let res = NewJobResult::send_results(
                                        //     &REPORT_CALLBACK,
                                        //     vec![job_result],
                                        // )
                                        // .await;
                                        // if res.is_err() {
                                        //     error!("send_results error: {:?}", res);
                                        // }
                                    }
                                    Err(error) => {
                                        error!("error: {:?}", error);
                                    }
                                };
                            }
                            Raw::Error(runtime_error) => {
                                error!("Some extrinsic Failed: {:?}", runtime_error);
                            }
                            _ => debug!("ignoring unsupported module event: {:?}", event),
                        }
                    }
                }
                Err(error) => error!("couldn't decode event record list: {:?}", error),
            }
            let res = NewJobResult::send_results(&REPORT_CALLBACK, vec_job_results).await;
            if res.is_err() {
                error!("send_results error: {:?}", res);
            }
        }
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

    use test_util::helper::{init_logging, load_env, mock_job, JobName};

    #[tokio::test]
    async fn test_call_chain() {
        load_env();
        init_logging();
        let job1 = mock_job(
            &JobName::RoundTripTime,
            "http://34.116.86.153/_rtt",
            "job_id_rtt1",
            &Default::default(),
        );
        let job2 = mock_job(
            &JobName::LatestBlock,
            "https://34.101.146.31",
            "job_id_latest_block",
            &Default::default(),
        );
        // Submit job
        let adapter = ChainAdapter::new();
        adapter
            .submit_jobs(&vec![job2, job1])
            .await
            .expect("submit_jobs error");
    }

    #[ignore]
    #[tokio::test]
    async fn test_listen_event() {
        load_env();
        init_logging();
        let adapter = ChainAdapter::new();

        // listen event
        //str::from_utf8()
        adapter.subscribe_event_onchain_worker().await;
    }
}
