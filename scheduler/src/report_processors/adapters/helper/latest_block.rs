use crate::LATEST_BLOCK_CACHING_DURATION;
use common::component::ChainInfo;

use common::job_manage::JobResultDetail;
use common::jobs::JobResult;
use common::tasks::http_request::{HttpResponseValues, JobHttpResponseDetail, JobHttpResult};
use common::util::{from_str_radix16, get_current_time};
use common::ComponentId;
pub use entity::seaorm::provider_latest_blocks::ActiveModel as ProviderLatestBlockActiveModel;
pub use entity::seaorm::provider_latest_blocks::Model as ProviderLatestBlockModel;
use log::debug;
use sea_orm::ActiveValue::Set;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct LatestBlockEntity {
    pub provider_id: String,
    pub blockchain: String,
    pub network: String,
    pub blockhash: Option<String>,
    pub block_timestamp: Option<i64>,
    pub max_block_timestamp: Option<i64>,
    pub block_number: Option<i64>,
    pub max_block_number: Option<i64>,
    pub response_timestamp: i64,
}

#[derive(Clone, Default, Debug)]
pub struct LatestBlockValue {
    pub block_timestamp: Option<i64>,
    pub block_number: Option<i64>,
    pub block_hash: Option<String>,
    pub response_timestamp: i64,
}

impl LatestBlockValue {
    pub fn is_empty(&self) -> bool {
        return self.block_number.is_none()
            && self.block_hash.is_none()
            && self.block_timestamp.is_none();
    }
}
impl From<HttpResponseValues> for LatestBlockValue {
    fn from(values: HttpResponseValues) -> Self {
        let block_number = values
            .get("number")
            .and_then(|val| val.as_str())
            .and_then(|str| from_str_radix16(str).ok());
        let block_timestamp = values
            .get("timestamp")
            .and_then(|val| val.as_str())
            .and_then(|str| from_str_radix16(str).ok());
        let block_hash = values
            .get("hash")
            .and_then(|val| val.as_str())
            .map(|val| val.to_string());
        LatestBlockValue {
            block_timestamp,
            block_number,
            block_hash,
            response_timestamp: 0,
        }
    }
}
#[derive(Clone, Default)]
pub struct LatestBlockCache {
    pub latest_flush_timestamp: i64,
    pub max_block_number: HashMap<ChainInfo, i64>,
    pub max_block_timestamp: HashMap<ChainInfo, i64>,
    pub latest_values: HashMap<ChainInfo, HashMap<ComponentId, LatestBlockValue>>,
}

impl LatestBlockCache {
    pub fn new() -> Self {
        Self {
            latest_flush_timestamp: get_current_time(),
            ..Default::default()
        }
    }
    pub fn create_active_model(model: &ProviderLatestBlockModel) -> ProviderLatestBlockActiveModel {
        ProviderLatestBlockActiveModel {
            provider_id: Set(model.provider_id.to_owned()),
            blockchain: Set(model.blockchain.to_owned()),
            network: Set(model.network.to_owned()),
            blockhash: Set(model.blockhash.to_owned()),
            block_timestamp: Set(model.block_timestamp.to_owned()),
            max_block_timestamp: Set(model.max_block_timestamp.to_owned()),
            block_number: Set(model.block_number.to_owned()),
            max_block_number: Set(model.max_block_number.to_owned()),
            response_timestamp: Set(model.response_timestamp.to_owned()),
            ..Default::default()
        }
    }
    pub fn append_result(&mut self, result: JobResult) {
        let JobResult {
            provider_id,
            result_detail,
            chain_info,
            ..
        } = result;
        if let (Some(chain_info), JobResultDetail::HttpRequest(JobHttpResult { response, .. })) =
            (chain_info, result_detail)
        {
            if response.error_code == 0 {
                if let JobHttpResponseDetail::Values(values) = response.detail {
                    let mut latest_value = LatestBlockValue::from(values.clone());
                    debug!(
                        "Latest value {:?} from response {:?}",
                        &latest_value, &values
                    );
                    if !latest_value.is_empty() {
                        latest_value.response_timestamp = response.request_timestamp;
                        self.add_latest_value(chain_info, provider_id, latest_value);
                    }
                }
            }
        }
    }
    pub fn add_latest_value(
        &mut self,
        chain_info: ChainInfo,
        provider_id: ComponentId,
        latest_value: LatestBlockValue,
    ) {
        let block_id = ChainInfo::new(chain_info.chain.clone(), chain_info.network.clone());
        if let Some(val) = latest_value.block_timestamp {
            let max = self
                .max_block_timestamp
                .get(&block_id)
                .map(|v| if *v < val { val } else { *v })
                .unwrap_or(val);
            self.max_block_timestamp.insert(block_id.clone(), max);
        }
        if let Some(val) = latest_value.block_number {
            let max = self
                .max_block_number
                .get(&block_id)
                .map(|v| if *v < val { val } else { *v })
                .unwrap_or(val);
            self.max_block_number.insert(block_id.clone(), max);
        }
        let map_latest_values = self
            .latest_values
            .entry(block_id)
            .or_insert_with(HashMap::default);
        map_latest_values.insert(provider_id, latest_value);
    }
    pub fn get_cache_data_for_flushing(&mut self) -> Vec<ProviderLatestBlockModel> {
        let mut result = Vec::new();
        let current_time = get_current_time();
        if current_time - self.latest_flush_timestamp >= *LATEST_BLOCK_CACHING_DURATION {
            self.latest_flush_timestamp = current_time;
            for (blockchain_id, latest_values) in self.latest_values.iter() {
                for (comp_id, value) in latest_values {
                    let model = ProviderLatestBlockModel {
                        id: 0,
                        provider_id: comp_id.clone(),
                        blockchain: blockchain_id.chain.to_string(),
                        network: blockchain_id.network.clone(),
                        blockhash: value.block_hash.clone(),
                        block_timestamp: value.block_timestamp.clone(),
                        max_block_timestamp: self
                            .max_block_timestamp
                            .get(blockchain_id)
                            .map(|val| val.clone()),
                        block_number: value.block_number.clone(),
                        max_block_number: self
                            .max_block_number
                            .get(blockchain_id)
                            .map(|val| val.clone()),
                        response_timestamp: value.response_timestamp.clone(),
                    };
                    result.push(model);
                }
            }
        }
        result
    }
}
