use crate::service::comparator::Comparator;
use anyhow::{anyhow, Error};
use common::tasks::http_request::HttpResponseValues;
use common::util::from_str_radix16;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct LatestBlockEthComparator {}

impl LatestBlockEthComparator {
    pub fn get_block_time(values: &HttpResponseValues) -> Result<i64, Error> {
        let timestamp = values
            .get("timestamp")
            .ok_or(anyhow!("timestamp not found"))
            .and_then(|val| val.as_str().ok_or(anyhow!("Invalid value")))
            .and_then(|val| from_str_radix16(val));
        timestamp
    }
}
impl Comparator for LatestBlockEthComparator {
    fn get_latest_value(&self, value: &HttpResponseValues) -> Option<i64> {
        self.get_number_value(value, "timestamp").ok()
    }
    fn compare(
        &self,
        value1: &HttpResponseValues,
        value2: &HttpResponseValues,
    ) -> Result<i64, Error> {
        if let (Ok(block_time1), Ok(block_time2)) =
            (Self::get_block_time(value1), Self::get_block_time(value2))
        {
            Ok(block_time1 - block_time2)
        } else {
            Err(anyhow!("Error parse timestamp"))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::service::comparator::LatestBlockEthComparator;
    use common::tasks::http_request::HttpResponseValues;
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn test_get_block_time() {
        let _comparator = LatestBlockEthComparator::default();
        let mut map_values = HashMap::<String, Value>::new();
        map_values.insert("number".to_string(), Value::from("0xe5a50f"));
        map_values.insert(
            "hash".to_string(),
            Value::from("0x5fe07f604a6ff02ab289f8ced5301dae2273104df3302b63d0c21b8eeeda6f75"),
        );
        map_values.insert("timestamp".to_string(), Value::from("0x62bd80c2"));
        let response_values = HttpResponseValues::new(map_values);
        let timestamp = LatestBlockEthComparator::get_block_time(&response_values);
        println!("{:?}", timestamp);
        assert_eq!(timestamp.ok(), Some(1656586434));
    }
}
