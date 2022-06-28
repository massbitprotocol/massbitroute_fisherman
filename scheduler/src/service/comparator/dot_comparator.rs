use crate::service::comparator::Comparator;
use common::tasks::http_request::HttpResponseValues;
use minifier::js::Keyword::Default;
use serde::{Deserialize, Serialize};
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct LatestBlockDotComparator {}

impl LatestBlockDotComparator {
    pub fn get_block_number(values: &HttpResponseValues) -> i64 {
        let number = values
            .get("number")
            .and_then(|val| val.as_str())
            .and_then(|str| i64::from_str_radix(str, 16).ok())
            .unwrap_or_default();
        number
    }
}
impl Comparator for LatestBlockDotComparator {
    fn get_latest_value(&self, value: &HttpResponseValues) -> Option<i64> {
        self.get_number_value(value, "number")
    }
    fn compare(&self, value1: &HttpResponseValues, value2: &HttpResponseValues) -> i64 {
        let number1 = Self::get_block_number(value1);
        let number2 = Self::get_block_number(value2);
        number1 - number2
    }
}
