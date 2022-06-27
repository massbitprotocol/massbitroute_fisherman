use crate::models::job_result::ProviderTask;
use crate::persistence::services::job_result_service::JobResultService;
use crate::service::comparator::{get_comparators, Comparator};
use crate::service::judgment::{JudgmentsResult, ReportCheck};
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use common::component::{ChainInfo, ComponentType};
use common::job_manage::{JobDetail, JobResultDetail, JobRole};
use common::jobs::{Job, JobResult};
use common::models::PlanEntity;
use common::tasks::eth::LatestBlockConfig;
use common::tasks::http_request::{
    HttpRequestJobConfig, HttpResponseValues, JobHttpResponseDetail, JobHttpResult,
};
use common::tasks::LoadConfig;
use common::{BlockChainType, NetworkType, Timestamp, WorkerId};
use diesel::IntoSql;
use log::{debug, info};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
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
    pub fn check_latest_block(
        &self,
        cache_key: CacheKey,
        result_value: ResultValue,
        comparator: Option<Arc<dyn Comparator>>,
        task_config: Option<HttpRequestJobConfig>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        let thresholds = task_config.map(|config| config.thresholds.clone());
        match cache_key.blockchain.as_str() {
            "eth" => self.check_latest_eth_block(cache_key, result_value, comparator, thresholds),
            "dot" => self.check_latest_dot_block(cache_key, result_value, comparator, thresholds),
            _ => Ok(JudgmentsResult::Unfinished),
        }
    }
    fn check_latest_eth_block(
        &self,
        cache_key: CacheKey,
        result_value: ResultValue,
        comparator: Option<Arc<dyn Comparator>>,
        thresholds: Option<Map<String, Value>>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        let latest_block_time = comparator
            .and_then(|comp| comp.get_latest_value(&result_value.values))
            .unwrap_or_default();
        let late_duration_ms = thresholds
            .and_then(|val| val["late_duration_ms"].as_i64())
            .unwrap_or_default();
        let late_duration = result_value.time - latest_block_time;
        info!(
            "execution_timestamp: {}, block_timestamp: {}, Latest block late_duration/threshold: {}s/{}s",
            result_value.time / 1000,
            latest_block_time/ 1000,
            late_duration / 1000,
            late_duration_ms / 1000,
        );

        if late_duration > late_duration_ms {
            Ok(JudgmentsResult::Failed)
        } else {
            Ok(JudgmentsResult::Pass)
        }
    }
    /*
     * Compare with max block number
     */
    fn check_latest_dot_block(
        &self,
        cache_key: CacheKey,
        result_value: ResultValue,
        comparator: Option<Arc<dyn Comparator>>,
        thresholds: Option<Map<String, Value>>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        let mut values = self.values.lock().unwrap();
        //Compare current value with max value in cache;
        let mut missing_block = i64::MIN;
        for (key, value) in values.iter() {
            let diff = comparator
                .as_ref()
                .map(|comparator| comparator.compare(&value.values, &result_value.values))
                .unwrap_or_default();
            if diff > missing_block {
                missing_block = diff
            }
        }
        //Block in current value newer then caches ones
        if missing_block < 0 {
            values.insert(cache_key.clone(), result_value);
        }
        let max_block_missing = thresholds
            .and_then(|val| val["max_block_missing"].as_i64())
            .unwrap_or_default();
        if missing_block < max_block_missing {
            Ok(JudgmentsResult::Pass)
        } else {
            Ok(JudgmentsResult::Failed)
        }
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
    task_configs: Vec<HttpRequestJobConfig>,
    result_service: Arc<JobResultService>,
    cache_values: LatestBlockResultCache,
    comparators: HashMap<String, Arc<dyn Comparator>>,
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
        let path = format!("{}/http_request.json", config_dir);
        let task_configs = HttpRequestJobConfig::read_config(path.as_str());
        let comparators = get_comparators();
        HttpLatestBlockJudgment {
            verification_config,
            regular_config,
            task_configs,
            result_service,
            cache_values: LatestBlockResultCache::default(),
            comparators,
        }
    }
    pub fn get_task_config(
        &self,
        phase: &JobRole,
        blockchain: &String,
        network: &String,
        provider_type: &ComponentType,
    ) -> Vec<HttpRequestJobConfig> {
        self.task_configs
            .iter()
            .filter(|config| {
                config.match_phase(phase)
                    && config.match_blockchain(blockchain)
                    && config.match_network(network)
                    && config.match_provider_type(&provider_type.to_string())
            })
            .map(|config| config.clone())
            .collect::<Vec<HttpRequestJobConfig>>()
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
        todo!()
    }
    /*
     * Job result received for each provider with task HttpRequest.LatestBlock
     */
    async fn apply_for_results(
        &self,
        provider_task: &ProviderTask,
        results: &Vec<JobResult>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        if results.is_empty() {
            return Ok(JudgmentsResult::Unfinished);
        }

        // all job results have the same phase
        let phase = results.first().unwrap().phase.clone();

        let config = match phase {
            JobRole::Verification => &self.verification_config,
            JobRole::Regular => &self.regular_config,
        };
        todo!()
        // let results = self.result_service.get_result_http_latest_blocks(job).await?;
        // info!("Latest block results: {:?}", results);
        // if results.is_empty() {
        //     return Ok(JudgmentsResult::Unfinished);
        // }
        // // Select result for judge
        // let result = results
        //     .iter()
        //     .max_by(|r1, r2| r1.execution_timestamp.cmp(&r2.execution_timestamp))
        //     .unwrap();
        //
        // // Get late duration from execution time to block time
        // if result.response.error_code != 0 {
        //     return Ok(JudgmentsResult::Error);
        // }
        //
        // let late_duration = result.execution_timestamp - result.response.block_timestamp * 1000;
        // info!(
        //     "execution_timestamp: {}, block_timestamp: {}, Latest block late_duration/threshold: {}s/{}s",
        //     result.execution_timestamp,
        //     result.response.block_timestamp * 1000,
        //     late_duration / 1000,
        //     config.late_duration_threshold_ms / 1000,
        // );
        //
        // return if (late_duration / 1000) > (config.late_duration_threshold_ms / 1000) {
        //     Ok(JudgmentsResult::Failed)
        // } else {
        //     Ok(JudgmentsResult::Pass)
        // };
    }
}
