use crate::models::job_result::ProviderTask;
use crate::persistence::services::job_result_service::JobResultService;
use crate::service::comparator::{get_comparators, Comparator, LatestBlockDefaultComparator};
use crate::service::judgment::{JudgmentsResult, ReportCheck};
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use common::component::ComponentType;
use common::job_manage::{JobResultDetail, JobRole};
use common::jobs::{Job, JobResult};
use common::models::PlanEntity;
use common::tasks::eth::LatestBlockConfig;
use common::tasks::http_request::{
    HttpRequestJobConfig, HttpResponseValues, JobHttpResponseDetail, JobHttpResult,
};
use common::tasks::{LoadConfig, TaskConfigTrait};
use common::{BlockChainType, ChainId, NetworkType, Timestamp};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
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
    pub fn from_job_result(job_result: &JobResult) -> Self {
        if let JobResultDetail::HttpRequest(JobHttpResult { response, .. }) =
            &job_result.result_detail
        {
            if let JobHttpResponseDetail::Values(values) = &response.detail {
                return ResultValue::new(response.response_duration.clone(), values.clone());
            }
        }
        ResultValue::default()
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
    // pub fn insert_values(&self, key: CacheKey, time: &Timestamp, values: &HashMap<String, Value>) {
    //     let mut map = self.values.lock().unwrap();
    //     debug!(
    //         "Insert result value {:?} with time {:?} to cache",
    //         values, time
    //     );
    //     let _blockchain = key.blockchain.as_str();
    //     let _network = key.network.as_str();
    //     match map.get(&key) {
    //         None => {
    //             map.insert(
    //                 key.clone(),
    //                 ResultValue::new(time.clone(), HttpResponseValues::new(values.clone())),
    //             );
    //         }
    //         Some(val) => {
    //             if val.time < *time {
    //                 map.insert(
    //                     key.clone(),
    //                     ResultValue::new(time.clone(), HttpResponseValues::new(values.clone())),
    //                 );
    //             }
    //         }
    //     }
    // }
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
        comparator: Arc<dyn Comparator>,
        thresholds: Map<String, Value>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        debug!(
            "Check latest block for blockchain {:?} with values {:?} and thresholds {:?}",
            &cache_key.blockchain, &result_value, &thresholds
        );
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
        comparator: Arc<dyn Comparator>,
        thresholds: Map<String, Value>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        let latest_block_time =
            comparator
                .get_latest_value(&result_value.values)
                .ok_or(anyhow!(
                    "Error: missing latest value in response {:?}",
                    &result_value.values
                ))?;

        let late_duration_threshold = thresholds
            .get("late_duration")
            .ok_or(anyhow!("Missing late_duration"))?
            .as_i64()
            .ok_or(anyhow!("Wrong value late_duration"))?;
        let late_duration = result_value.time / 1000 - latest_block_time; //In seconds
        info!(
            "execution_timestamp: {}, block_timestamp: {}, Latest block late_duration/threshold: {}s/{}s",
            result_value.time / 1000,
            latest_block_time,
            late_duration,
            late_duration_threshold,
        );

        if late_duration > late_duration_threshold {
            info!("Judge Failed latest-block for Node eth {:?}", cache_key);
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
        comparator: Arc<dyn Comparator>,
        thresholds: Map<String, Value>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        let mut values = self.values.lock().unwrap();
        //Compare current value with max value in cache;
        let mut missing_block = i64::MIN;
        for (_key, value) in values.iter() {
            let diff = comparator.compare(&value.values, &result_value.values)?;
            if diff > missing_block {
                missing_block = diff
            }
        }
        //Block in current value newer then caches ones
        if missing_block < 0 {
            values.insert(cache_key.clone(), result_value);
        }
        let max_block_missing = thresholds
            .get("max_block_missing")
            .ok_or(anyhow!("Missing max_block_missing"))?
            .as_i64()
            .ok_or(anyhow!("Wrong value max_block_missing"))?;

        if missing_block < max_block_missing {
            Ok(JudgmentsResult::Pass)
        } else {
            info!("Judge Failed latest-block for Node Dot {:?}", cache_key);
            Ok(JudgmentsResult::Failed)
        }
    }
    /*
     * get all latest block values from current cache
     */
    // pub fn get_cache_values(&self, key: &CacheKey) -> Vec<ResultValue> {
    //     let blockchain = key.blockchain.as_str();
    //     let network = key.network.as_str();
    //     self.values
    //         .lock()
    //         .unwrap()
    //         .iter()
    //         .filter(|(k, _)| k.blockchain.as_str() == blockchain && k.network.as_str() == network)
    //         .map(|(_, v)| v.clone())
    //         .collect::<Vec<ResultValue>>()
    // }
}

#[derive(Debug)]
pub struct HttpLatestBlockJudgment {
    verification_config: LatestBlockConfig,
    regular_config: LatestBlockConfig,
    task_configs: Vec<HttpRequestJobConfig>,
    result_service: Arc<JobResultService>,
    cache_values: LatestBlockResultCache,
    comparators: HashMap<ChainId, Arc<dyn Comparator>>,
}

impl HttpLatestBlockJudgment {
    pub fn new(config_dir: &str, phase: &JobRole, result_service: Arc<JobResultService>) -> Self {
        let verification_config = LatestBlockConfig::load_config(
            format!("{}/latest_block.json", config_dir).as_str(),
            &JobRole::Verification,
        );
        let regular_config = LatestBlockConfig::load_config(
            format!("{}/latest_block.json", config_dir).as_str(),
            &JobRole::Regular,
        );
        let path = format!("{}/http_request.json", config_dir);
        let task_configs = HttpRequestJobConfig::read_config(path.as_str(), phase);
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
    ) -> Map<String, Value> {
        self.task_configs
            .iter()
            .filter(|config| {
                config.match_phase(phase)
                    && config.match_blockchain(blockchain)
                    && config.match_network(network)
                    && config.match_provider_type(&provider_type.to_string())
                    && config.name.as_str() == "LatestBlock"
            })
            .map(|config| config.clone())
            .collect::<Vec<HttpRequestJobConfig>>()
            .get(0)
            .map(|config| config.thresholds.clone())
            .unwrap_or_default()
    }
    pub fn get_comparator(&self, chain_id: &ChainId) -> Arc<dyn Comparator> {
        self.comparators
            .get(chain_id)
            .map(|item| item.clone())
            .unwrap_or(Arc::new(LatestBlockDefaultComparator::default()))
    }
}

#[async_trait]
impl ReportCheck for HttpLatestBlockJudgment {
    fn get_name(&self) -> String {
        String::from("HttpLatestBlock")
    }
    fn can_apply_for_result(&self, task: &ProviderTask) -> bool {
        return task.task_type.as_str() == "HttpRequest"
            && task.task_name.as_str() == "LatestBlock";
    }
    // async fn apply(&self, _plan: &PlanEntity, _job: &Vec<Job>) -> Result<JudgmentsResult, Error> {
    //     //Todo: remove this function
    //     Ok(JudgmentsResult::Error)
    //     /*
    //     let config = match JobRole::from_str(&*plan.phase)? {
    //         JobRole::Verification => &self.verification_config,
    //         JobRole::Regular => &self.regular_config,
    //     };
    //
    //     let results = self.result_service.get_result_latest_blocks(job).await?;
    //     info!("Latest block results: {:?}", results);
    //     if results.is_empty() {
    //         return Ok(JudgmentsResult::Unfinished);
    //     }
    //     // Select result for judge
    //     let result = results
    //         .iter()
    //         .max_by(|r1, r2| r1.execution_timestamp.cmp(&r2.execution_timestamp))
    //         .unwrap();
    //     // Get late duration from execution time to block time
    //     if result.response.error_code != 0 {
    //         return Ok(JudgmentsResult::Error);
    //     }
    //     let late_duration = result.execution_timestamp - result.response.block_timestamp * 1000;
    //     info!(
    //         "execution_timestamp: {}, block_timestamp: {}, Latest block late_duration/threshold: {}s/{}s",
    //         result.execution_timestamp,
    //         result.response.block_timestamp * 1000,
    //         late_duration / 1000,
    //         config.late_duration_threshold_ms / 1000,
    //     );
    //
    //     return if (late_duration / 1000) > (config.late_duration_threshold_ms / 1000) {
    //         Ok(JudgmentsResult::Failed)
    //     } else {
    //         Ok(JudgmentsResult::Pass)
    //     };
    //      */
    // }
    /*
     * Job result received for each provider with task HttpRequest.LatestBlock
     */
    async fn apply_for_results(
        &self,
        provider_task: &ProviderTask,
        job_results: &Vec<JobResult>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        if job_results.is_empty() {
            return Ok(JudgmentsResult::Unfinished);
        }
        // Get comparator for the fist item
        let first_result = job_results.first().unwrap();
        let chain_info = first_result
            .chain_info
            .as_ref()
            .ok_or(anyhow!("Missing chain_info"))?
            .clone();
        let comparator = self.get_comparator(&chain_info.chain);

        //Filter and get only latest result to check

        let cache_key = CacheKey::new(
            provider_task.provider_id.clone(),
            chain_info.chain.clone(),
            chain_info.network.clone(),
        );
        let mut latest_job_result = first_result;
        let mut latest_result_values = ResultValue::from_job_result(first_result);
        debug!("Latest result values {:?}", &latest_result_values);
        // Get newest result from cache
        for result in job_results {
            if result.chain_info.is_none() {
                return Ok(JudgmentsResult::Error);
            }

            if let (
                JobResultDetail::HttpRequest(latest_detail),
                JobResultDetail::HttpRequest(current_detail),
            ) = (&latest_job_result.result_detail, &result.result_detail)
            {
                if let (
                    JobHttpResponseDetail::Values(latest_values),
                    JobHttpResponseDetail::Values(current_values),
                ) = (
                    &latest_detail.response.detail,
                    &current_detail.response.detail,
                ) {
                    if let Ok(diff) = comparator.compare(&latest_values, current_values) {
                        if diff < 0 {
                            latest_result_values = ResultValue::new(
                                current_detail.response.response_duration.clone(),
                                current_values.clone(),
                            );
                            latest_job_result = result;
                        }
                    }
                }
            }
        }
        // Get threshold from config
        println!(
            "cache_key: {:?}, latest_job_result: {:?}",
            cache_key, latest_job_result
        );
        let thresholds = self.get_task_config(
            &latest_job_result.phase,
            &cache_key.blockchain,
            &cache_key.network,
            &latest_job_result.provider_type,
        );

        println!("get_task_config thresholds: {:?}", thresholds);

        // Get threshold from config
        let res = self.cache_values.check_latest_block(
            cache_key,
            latest_result_values,
            comparator,
            thresholds,
        );
        info!("Judg {:?} latest-block res: {:?}", provider_task, res);
        res
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::CONFIG_DIR;

    use test_util::helper::{
        init_logging, load_env, mock_db_connection, mock_job_result, ChainTypeForTest, JobName,
    };

    #[tokio::test]
    async fn test_http_latest_block_judgment() -> Result<(), Error> {
        load_env();
        //init_logging();
        let db_conn = mock_db_connection();
        let result_service = JobResultService::new(Arc::new(db_conn));
        let judge = HttpLatestBlockJudgment::new(
            CONFIG_DIR.as_str(),
            &JobRole::Verification,
            Arc::new(result_service),
        );

        println!("Task configs: {:?}", judge.task_configs);
        let task_latest_block = ProviderTask::new(
            "provider_id".to_string(),
            ComponentType::Node,
            "HttpRequest".to_string(),
            "LatestBlock".to_string(),
        );
        let task_rtt = ProviderTask::new(
            "provider_id".to_string(),
            ComponentType::Node,
            "HttpRequest".to_string(),
            "RoundTripTime".to_string(),
        );

        // Test can_apply_for_result
        assert!(judge.can_apply_for_result(&task_latest_block));
        assert!(!judge.can_apply_for_result(&task_rtt));

        // Test apply_for_results
        assert_eq!(
            judge.apply_for_results(&task_latest_block, &vec![]).await?,
            JudgmentsResult::Unfinished
        );

        // For eth
        let job_result = mock_job_result(
            &JobName::LatestBlock,
            ChainTypeForTest::Eth,
            "",
            Default::default(),
        );
        info!("job_result: {:?}", job_result);
        let res = judge
            .apply_for_results(&task_latest_block, &vec![job_result])
            .await?;
        println!("Judge res: {:?}", res);
        assert_eq!(res, JudgmentsResult::Pass);

        // For dot
        let job_result = mock_job_result(
            &JobName::LatestBlock,
            ChainTypeForTest::Dot,
            "",
            Default::default(),
        );
        info!("job_result: {:?}", job_result);
        let res = judge
            .apply_for_results(&task_latest_block, &vec![job_result])
            .await?;
        println!("Judge res: {:?}", res);
        assert_eq!(res, JudgmentsResult::Pass);

        Ok(())
    }
}
