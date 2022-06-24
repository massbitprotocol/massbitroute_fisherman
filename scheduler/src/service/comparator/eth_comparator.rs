use crate::service::comparator::Comparator;
use common::tasks::http_request::HttpResponseValues;
use serde::{Deserialize, Serialize};
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct LatestBlockEthComparator {}

impl LatestBlockEthComparator {
    pub fn get_block_time(values: &HttpResponseValues) -> u64 {
        todo!()
    }
}
impl Comparator for LatestBlockEthComparator {
    fn compare(&self, value1: &HttpResponseValues, value2: &HttpResponseValues) -> i64 {
        // let block_timestamp = block_timestamp
        //     .trim_start_matches("\"0x")
        //     .trim_end_matches("\"");
        // info!(
        //     "result: {}, block_hash: {}, block_number: {}, block_timestamp: {}",
        //     result, block_hash, block_number, block_timestamp
        // );
        //
        // Ok(BlockData {
        //     block_number: u64::from_str_radix(block_number, 16)?,
        //     block_timestamp: i64::from_str_radix(block_timestamp, 16)?,
        //     block_hash,
        // })
        todo!()
    }
}
