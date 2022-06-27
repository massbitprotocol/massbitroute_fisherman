use crate::service::comparator::Comparator;
use common::tasks::http_request::HttpResponseValues;
use common::util::get_current_time;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct LatestBlockEthComparator {}

impl LatestBlockEthComparator {
    pub fn get_block_time(values: &HttpResponseValues) -> i64 {
        let timestamp = values
            .get("timestamp")
            .and_then(|val| val.as_str())
            .and_then(|str| i64::from_str_radix(str, 16).ok())
            .unwrap_or_default();
        timestamp
    }
}
impl Comparator for LatestBlockEthComparator {
    fn get_latest_value(&self, value: &HttpResponseValues) -> Option<i64> {
        self.get_number_value(value, "timestamp")
    }
    fn compare(&self, value1: &HttpResponseValues, value2: &HttpResponseValues) -> i64 {
        let block_time1 = Self::get_block_time(value1);
        let block_time2 = Self::get_block_time(value2);
        block_time1 - block_time2
    }
}