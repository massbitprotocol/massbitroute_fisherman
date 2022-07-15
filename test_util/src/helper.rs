use crate::helper::JobName::Benchmark;
use common::component::{ChainInfo, ComponentType, Zone};
use common::job_manage::{
    BenchmarkResponse, JobBenchmark, JobBenchmarkResult, JobDetail, JobResultDetail, JobRole,
};
use common::jobs::{Job, JobResult};
use common::logger::init_logger;
use common::tasks::eth::JobLatestBlock;
use common::tasks::http_request::{
    HttpResponseValues, JobHttpRequest, JobHttpResponse, JobHttpResponseDetail, JobHttpResult,
};
use common::util::get_current_time;
use common::workers::WorkerInfo;
use common::{ChainId, ComponentInfo};
use entity::job_result_http_requests;
use sea_orm::{DatabaseBackend, DatabaseConnection, MockDatabase, MockExecResult};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, AtomicUsize};

static GLOBAL_INIT_LOGGING: AtomicBool = AtomicBool::new(false);

pub fn load_env() {
    dotenv::from_filename(".env_test").ok();
}

pub fn init_logging() {
    let _res = init_logger(&String::from("Testing"));
}

pub enum JobName {
    RoundTripTime,
    LatestBlock,
    Benchmark,
}

pub enum ChainTypeForTest {
    Eth,
    Dot,
}

impl ToString for ChainTypeForTest {
    fn to_string(&self) -> String {
        match self {
            ChainTypeForTest::Eth => "eth".to_string(),
            ChainTypeForTest::Dot => "dot".to_string(),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct CountItems<T: Hash + Eq> {
    inner: HashMap<T, u64>,
}

impl<T: Hash + Eq + Debug> CountItems<T> {
    pub fn add_item(&mut self, item: T) {
        let entry = self.inner.entry(item).or_insert(0);
        *entry += 1;
    }
    pub fn add_items(&mut self, items: Vec<T>) {
        for item in items {
            self.add_item(item);
        }
    }
    pub fn new(items: Vec<T>) -> Self {
        let mut res = CountItems {
            inner: HashMap::new(),
        };
        res.add_items(items);
        res
    }
    pub fn sum_len(&self) -> u64 {
        self.inner.values().into_iter().fold(0, |sum, &x| sum + x)
    }
}

pub fn mock_worker(id: &str) -> WorkerInfo {
    WorkerInfo {
        worker_id: id.to_string(),
        worker_ip: "2.2.2.2".to_string(),
        url: "2.2.2.2:2".to_string(),
        zone: Zone::AS,
        worker_spec: Default::default(),
        available_time_frame: None,
    }
}

pub fn mock_db_connection() -> DatabaseConnection {
    let data = job_result_http_requests::Model {
        id: 1,
        job_id: "".to_string(),
        job_name: "".to_string(),
        worker_id: "".to_string(),
        provider_id: "".to_string(),
        provider_type: "".to_string(),
        execution_timestamp: 0,
        chain_id: "".to_string(),
        plan_id: "".to_string(),
        http_code: 0,
        error_code: 0,
        message: "".to_string(),
        values: Default::default(),
        response_duration: 0,
    };
    let exec_res = MockExecResult {
        last_insert_id: 1 as u64,
        rows_affected: 1,
    };

    let db_conn = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results(vec![vec![data; 1000]])
        .append_exec_results(vec![exec_res; 1000])
        .into_connection();
    db_conn
}

pub fn mock_component_info(
    id: &str,
    chain: &ChainTypeForTest,
    component_type: &ComponentType,
) -> ComponentInfo {
    ComponentInfo {
        blockchain: chain.to_string(),
        network: "main".to_string(),
        id: id.to_string(),
        user_id: "user_id".to_string(),
        ip: "1.1.1.1".to_string(),
        zone: Zone::AS,
        country_code: "US".to_string(),
        token: "token".to_string(),
        component_type: component_type.clone(),
        endpoint: None,
        status: "stacked".to_string(),
    }
}

pub fn mock_job_result(
    job_name: &JobName,
    chain: ChainTypeForTest,
    job_id: &str,
    phase: JobRole,
) -> JobResult {
    let job = mock_job(job_name, "", job_id, &phase);
    let chain_info = match chain {
        ChainTypeForTest::Eth => ChainInfo {
            chain: "eth".to_string(),
            network: "main".to_string(),
        },
        ChainTypeForTest::Dot => ChainInfo {
            chain: "dot".to_string(),
            network: "main".to_string(),
        },
    };

    let job_result_detail = match job_name {
        JobName::Benchmark => {
            let mut resp = BenchmarkResponse {
                histograms: HashMap::from([
                    (90, 300f32),
                    (95, 300f32),
                    (99, 300f32),
                    (100, 300f32),
                ]),
                ..Default::default()
            };
            JobResultDetail::Benchmark(JobBenchmarkResult {
                job: job.clone(),
                worker_id: "".to_string(),
                response_timestamp: get_current_time(),
                response: resp,
            })
        }
        JobName::LatestBlock => {
            let detail: JobHttpResponseDetail = match chain {
                ChainTypeForTest::Eth => {
                    serde_json::from_str(r###"
            {"Values": {"inner": {"hash": "0x7e915fa20e34a184701607091cf6715744889751b9485aae7b04ef165aa6cacc", "number": "0xe5a51a", "timestamp": "0x62c217d5"}}}
            "###).unwrap()
                }
                ChainTypeForTest::Dot => {
                    serde_json::from_str(r###"
            {"Values": {"inner": {"number": "0xa88c38", "parent_hash": "0x9dc0f5d6d7e25e2b9c6108c57c04daaf63913c71f706ab17ac4f21c58df674e1"}}}
            "###).unwrap()
                }
            };
            let resp = JobHttpResponse {
                detail,
                ..Default::default()
            };

            JobResultDetail::HttpRequest(JobHttpResult::new(job.clone(), resp))
        }
        JobName::RoundTripTime => {
            let detail = JobHttpResponseDetail::Body("175824".to_string());
            let resp = JobHttpResponse {
                detail,
                ..Default::default()
            };
            JobResultDetail::HttpRequest(JobHttpResult::new(job.clone(), resp))
        }
    };
    let mut job_result = JobResult::new(job_result_detail, Some(chain_info), &job);
    println!("job_result: {:?}", job_result);
    job_result
}

pub fn mock_job(job_name: &JobName, component_url: &str, job_id: &str, phase: &JobRole) -> Job {
    let component = ComponentInfo {
        blockchain: "".to_string(),
        network: "".to_string(),
        id: job_id.to_string(),
        user_id: "".to_string(),
        ip: "".to_string(),
        zone: Default::default(),
        country_code: "".to_string(),
        token: "".to_string(),
        component_type: Default::default(),
        endpoint: None,
        status: "".to_string(),
    };

    let mut job = match job_name {
        JobName::Benchmark => {
            let job_detail = r###"
        {"rate": 50, "script": "massbit.lua", "thread": 20, "duration": 15000, "url_path": "/", "chain_type": "eth", "connection": 20, "histograms": [90, 95, 99, 100], "component_type": "Node"}
        "###;
            let job_detail: JobBenchmark = serde_json::from_str(job_detail).unwrap();
            Job::new(
                "benchmark".to_string(),
                "Benchmark".to_string(),
                "Benchmark".to_string(),
                &component,
                JobDetail::Benchmark(job_detail),
                phase.clone(),
            )
        }
        JobName::RoundTripTime => {
            let job_detail = r###"
        {"url": "", "body": "", "method": "get", "headers": {}, "chain_info": {"chain": "eth", "network": "mainnet"}, "response_type": "text", "response_values": {}}
        "###;
            let mut job_http_request: JobHttpRequest = serde_json::from_str(job_detail).unwrap();
            job_http_request.url = component_url.to_string();
            Job::new(
                "http".to_string(),
                "HttpRequest".to_string(),
                "RoundTripTime".to_string(),
                &component,
                JobDetail::HttpRequest(job_http_request),
                phase.clone(),
            )
        }
        JobName::LatestBlock => {
            let job_detail = r###"
        {"url": "https://67.219.104.215/", "body": {"id": 1, "method": "eth_getBlockByNumber", "params": ["latest", true], "jsonrpc": "2.0"}, "method": "post", "headers": {"Host": "6e02171a-93b0-4079-91b6-caddc64f5dbc.node.mbr.massbitroute.net", "X-Api-Key": "rLhwVAprTNK8yqYmqSmXug", "content-type": "application/json"}, "chain_info": {"chain": "eth", "network": "mainnet"}, "response_type": "json", "response_values": {"hash": ["result", "hash"], "number": ["result", "number"], "timestamp": ["result", "timestamp"]}}
        "###;
            let job_latest_block: JobHttpRequest = serde_json::from_str(job_detail).unwrap();
            Job::new(
                "http".to_string(),
                "HttpRequest".to_string(),
                "RoundTripTime".to_string(),
                &component,
                JobDetail::HttpRequest(job_latest_block),
                phase.clone(),
            )
        }
    };
    job.component_url = component_url.to_string();
    job.job_id = job_id.to_string();
    job
}
