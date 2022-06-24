use crate::service::comparator::Comparator;
use common::tasks::http_request::HttpResponseValues;
use serde::{Deserialize, Serialize};
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct LatestBlockDotComparator {}

impl Comparator for LatestBlockDotComparator {
    fn compare(&self, value1: &HttpResponseValues, value2: &HttpResponseValues) -> i64 {
        todo!()
    }
}
