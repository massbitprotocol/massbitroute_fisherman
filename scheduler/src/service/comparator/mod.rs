mod default_comparator;
mod dot_comparator;
mod eth_comparator;
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use common::tasks::http_request::HttpResponseValues;
use common::util::from_str_radix16;
pub use default_comparator::LatestBlockDefaultComparator;
pub use dot_comparator::LatestBlockDotComparator;
pub use eth_comparator::LatestBlockEthComparator;
use log::debug;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait Comparator: Sync + Send + Debug {
    fn get_number_value(&self, value: &HttpResponseValues, field: &str) -> Result<i64, Error> {
        let res = value
            .get(field)
            .ok_or_else(|| anyhow!("Field {} not found", field))
            .and_then(|val| val.as_str().ok_or_else(|| anyhow!("Invalid value")))
            .and_then(from_str_radix16);
        debug!("Get field {} from {:?} return {:?}", field, value, &res);
        res
    }
    fn get_latest_value(&self, value: &HttpResponseValues) -> Option<i64>;
    fn compare(
        &self,
        value1: &HttpResponseValues,
        value2: &HttpResponseValues,
    ) -> Result<i64, Error>;
}

pub fn get_comparators() -> HashMap<String, Arc<dyn Comparator>> {
    let mut comparators = HashMap::<String, Arc<dyn Comparator>>::new();
    comparators.insert(
        String::from("eth"),
        Arc::new(LatestBlockEthComparator::default()),
    );
    comparators.insert(
        String::from("dot"),
        Arc::new(LatestBlockDotComparator::default()),
    );
    comparators
}
