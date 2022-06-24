use crate::models::job_result::ProviderTask;
use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::{JudgmentsResult, ReportCheck};
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use common::component::ChainInfo;
use common::job_manage::{JobDetail, JobResultDetail, JobRole};
use common::jobs::{Job, JobResult};
use common::models::PlanEntity;
use common::tasks::eth::LatestBlockConfig;
use common::tasks::http_request::{HttpResponseValues, JobHttpResponseDetail, JobHttpResult};
use common::tasks::LoadConfig;
use common::{BlockChainType, NetworkType, Timestamp, WorkerId};
use log::{debug, info};
use minifier::js::Keyword::Default;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash, Default)]
pub struct CacheKey {
    pub blockchain: BlockChainType,
    pub network: NetworkType,
    pub provider_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct ResultValue {
    pub time: Timestamp,
    pub values: HttpResponseValues,
}

impl ResultValue {
    pub fn new(time: Timestamp, values: HttpResponseValues) -> Self {
        Self { time, values }
    }
}
impl CacheKey {
    pub fn new(provider_id: String, blockchain: BlockChainType, network: NetworkType) -> Self {
        Self {
            provider_id,
            blockchain,
            network,
        }
    }
}
#[derive(Debug, Default)]
pub struct LatestBlockResultCache {
    values: Mutex<HashMap<CacheKey, ResultValue>>,
}

impl LatestBlockResultCache {
    pub fn insert_values(&self, key: CacheKey, time: &Timestamp, values: &HashMap<String, Value>) {
        let mut map = self.values.lock().unwrap();
        //Check if current values is newer than the one in cache
        let blockchain = key.blockchain.as_str();
        let network = key.network.as_str();
        match map.get(&key) {
            None => {
                map.insert(
                    key.clone(),
                    ResultValue::new(time.clone(), HttpResponseValues::new(values.clone())),
                );
            }
            Some(val) => {
                if val.time < *time {
                    map.insert(
                        key.clone(),
                        ResultValue::new(time.clone(), HttpResponseValues::new(values.clone())),
                    );
                }
            }
        }
    }
    /*
     * Compare received latest values with cached values.
     * Hard code some parser function base on blockchain
     * Use check function here to avoid clone data
     * return true if value is up to date and put it back to cache
     */
    pub fn check_latest_block(&self, cache_key: &CacheKey, result_value: ResultValue) -> bool {
        match cache_key.blockchain.as_str() {
            "eth" => self.check_latest_eth_block(result_value),
            "dot" => self.check_latest_dot_block(result_value),
            _ => true,
        }
    }
    fn check_latest_eth_block(&self, result_value: ResultValue) -> bool {
        true
    }
    fn check_latest_dot_block(&self, result_value: ResultValue) -> bool {
        true
    }
    /*
     * get all latest block values from current cache
     */
    pub fn get_cache_values(&self, key: &CacheKey) -> Vec<ResultValue> {
        let blockchain = key.blockchain.as_str();
        let network = key.network.as_str();
        self.values
            .lock()
            .unwrap()
            .iter()
            .filter(|(k, _)| k.blockchain.as_str() == blockchain && k.network.as_str() == network)
            .map(|(_, v)| v.clone())
            .collect::<Vec<ResultValue>>()
    }
}
#[derive(Debug)]
pub struct HttpLatestBlockJudgment {
    verification_config: LatestBlockConfig,
    regular_config: LatestBlockConfig,
    result_service: Arc<JobResultService>,
    cache_values: LatestBlockResultCache,
}

impl HttpLatestBlockJudgment {
    pub fn new(config_dir: &str, result_service: Arc<JobResultService>) -> Self {
        let verification_config = LatestBlockConfig::load_config(
            format!("{}/latest_block.json", config_dir).as_str(),
            &JobRole::Verification,
        );
        let regular_config = LatestBlockConfig::load_config(
            format!("{}/latest_block.json", config_dir).as_str(),
            &JobRole::Regular,
        );
        HttpLatestBlockJudgment {
            verification_config,
            regular_config,
            result_service,
            cache_values: LatestBlockResultCache::default(),
        }
    }
}

#[async_trait]
impl ReportCheck for HttpLatestBlockJudgment {
    fn can_apply(&self, job: &Job) -> bool {
        match job.job_name.as_str() {
            "LatestBlock" => true,
            _ => false,
        }
    }
    fn can_apply_for_result(&self, task: &ProviderTask) -> bool {
        return task.task_type.as_str() == "HttpRequest"
            && task.task_name.as_str() == "LatestBlock";
    }
    async fn apply(&self, plan: &PlanEntity, job: &Job) -> Result<JudgmentsResult, Error> {
        let config = match JobRole::from_str(&*plan.phase)? {
            JobRole::Verification => &self.verification_config,
            JobRole::Regular => &self.regular_config,
        };

        let results = self.result_service.get_result_latest_blocks(job).await?;
        info!("Latest block results: {:?}", results);
        if results.is_empty() {
            return Ok(JudgmentsResult::Unfinished);
        }
        // Select result for judge
        let result = results
            .iter()
            .max_by(|r1, r2| r1.execution_timestamp.cmp(&r2.execution_timestamp))
            .unwrap();
        // Get late duration from execution time to block time
        if result.response.error_code != 0 {
            return Ok(JudgmentsResult::Error);
        }
        let late_duration = result.execution_timestamp - result.response.block_timestamp * 1000;
        info!(
            "execution_timestamp: {}, block_timestamp: {}, Latest block late_duration/threshold: {}s/{}s",
            result.execution_timestamp,
            result.response.block_timestamp * 1000,
            late_duration / 1000,
            config.late_duration_threshold_ms / 1000,
        );

        return if (late_duration / 1000) > (config.late_duration_threshold_ms / 1000) {
            Ok(JudgmentsResult::Failed)
        } else {
            Ok(JudgmentsResult::Pass)
        };
    }
    /*
     * Job result received for each provider with task HttpRequest.LatestBlock
     */
    async fn apply_for_results(
        &self,
        provider_task: &ProviderTask,
        job_results: &Vec<JobResult>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        //Filter and get only latest result to check
        let mut latest_values = ResultValue::default();
        let mut cache_key = CacheKey::new(
            provider_task.provider_id.clone(),
            BlockChainType::new(),
            NetworkType::new(),
        );
        for result in job_results {
            if let JobResultDetail::HttpRequest(JobHttpResult { response, .. }) =
                &result.result_detail
            {
                if response.response_time <= latest_values.time {
                    continue;
                }
                if let JobHttpResponseDetail::Values(values) = &response.detail {
                    latest_values =
                        ResultValue::new(response.response_time.clone(), values.clone());
                    let (blockchain, network) = result
                        .chain_info
                        .as_ref()
                        .map(|info| (info.chain.clone(), info.network.clone()))
                        .unwrap_or_default();
                    cache_key.blockchain = blockchain;
                    cache_key.network = network;
                }
            }
        }

        let check_res = self
            .cache_values
            .check_latest_block(&cache_key, latest_values);
        if check_res {
            Ok(JudgmentsResult::Pass)
        } else {
            Ok(JudgmentsResult::Failed)
        }
    }
}
