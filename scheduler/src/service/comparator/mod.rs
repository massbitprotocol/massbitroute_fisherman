mod dot_comparator;
mod eth_comparator;
use async_trait::async_trait;
use common::tasks::http_request::HttpResponseValues;
pub use dot_comparator::LatestBlockDotComparator;
pub use eth_comparator::LatestBlockEthComparator;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
#[async_trait]
pub trait Comparator: Sync + Send + Debug {
    fn compare(&self, value1: &HttpResponseValues, value2: &HttpResponseValues) -> i64;
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
