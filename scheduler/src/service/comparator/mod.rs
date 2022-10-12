mod default_comparator;
mod dot_comparator;
mod eth_comparator;
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use common::tasks::http_request::HttpResponseValues;
use common::util::from_str_radix16;
use common::BlockChainType;
pub use default_comparator::LatestBlockDefaultComparator;
pub use dot_comparator::LatestBlockDotComparator;
pub use eth_comparator::LatestBlockEthComparator;
use log::{debug, trace};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait Comparator: Sync + Send + Debug {
    fn get_number_value(&self, value: &HttpResponseValues, field: &str) -> Result<i64, Error> {
        let res = value
            .get(field)
            .ok_or(anyhow!("Field {} not found", field))
            .and_then(|val| val.as_str().ok_or(anyhow!("Invalid value")))
            .and_then(|str| from_str_radix16(str));
        trace!("Get field {} from {:?} return {:?}", field, value, &res);
        res
    }
    fn get_latest_value(&self, value: &HttpResponseValues) -> Option<i64>;
    fn compare(
        &self,
        value1: &HttpResponseValues,
        value2: &HttpResponseValues,
    ) -> Result<i64, Error>;
}

pub fn get_comparators() -> HashMap<BlockChainType, Arc<dyn Comparator>> {
    let mut comparators = HashMap::<BlockChainType, Arc<dyn Comparator>>::new();
    comparators.insert(
        BlockChainType::Eth,
        Arc::new(LatestBlockEthComparator::default()),
    );
    comparators.insert(
        BlockChainType::Matic,
        Arc::new(LatestBlockEthComparator::default()),
    );
    comparators.insert(
        BlockChainType::Bsc,
        Arc::new(LatestBlockEthComparator::default()),
    );
    comparators.insert(
        BlockChainType::Dot,
        Arc::new(LatestBlockDotComparator::default()),
    );
    comparators
}
