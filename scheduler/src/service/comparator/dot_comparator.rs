use crate::service::comparator::Comparator;
use anyhow::{anyhow, Error};
use common::tasks::http_request::HttpResponseValues;
use serde::{Deserialize, Serialize};
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct LatestBlockDotComparator {}

impl LatestBlockDotComparator {
    pub fn get_block_number(values: &HttpResponseValues) -> Result<i64, Error> {
        let timestamp = values
            .get("number")
            .and_then(|val| val.as_str())
            .and_then(|string| {
                i64::from_str_radix(string.strip_prefix("0x").unwrap_or(string), 16).ok()
            })
            .ok_or_else(|| anyhow!("Error parse Dot block_number"));
        timestamp
    }
}
impl Comparator for LatestBlockDotComparator {
    fn get_latest_value(&self, value: &HttpResponseValues) -> Option<i64> {
        self.get_number_value(value, "number").ok()
    }
    fn compare(
        &self,
        value1: &HttpResponseValues,
        value2: &HttpResponseValues,
    ) -> Result<i64, Error> {
        if let (Ok(block_time1), Ok(block_time2)) = (
            Self::get_block_number(value1),
            Self::get_block_number(value2),
        ) {
            Ok(block_time1 - block_time2)
        } else {
            Err(anyhow!("Error Dot parse block_number"))
        }
    }
}
