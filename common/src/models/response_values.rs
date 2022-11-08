use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct ResponseConfig {
    #[serde(default)]
    pub response_type: String, //Response type: json or text
    #[serde(default)]
    pub values: HashMap<String, Vec<Value>>, //Path to values
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ResponseValues {
    inner: HashMap<String, Value>,
}

impl ResponseValues {
    pub fn new(inner: HashMap<String, Value>) -> Self {
        ResponseValues { inner }
    }
    pub fn extract_values(
        content: &str,
        value_paths: &HashMap<String, Vec<Value>>,
    ) -> Result<ResponseValues, anyhow::Error> {
        let body: Value = serde_json::from_str(content)
            .map_err(|e| anyhow!("Err {} when parsing response", e))?;
        let mut results = ResponseValues::default();
        for (key, paths) in value_paths.iter() {
            let mut ind = 0_usize;
            let mut tmp_value = &body;
            while ind < paths.len() {
                let field: &Value = paths.get(ind).unwrap();
                if field.is_string() && tmp_value.is_object() {
                    tmp_value = &tmp_value[field.as_str().unwrap()]
                } else if field.is_number() && tmp_value.is_array() {
                    tmp_value = &tmp_value[field.as_u64().unwrap() as usize]
                }
                ind += 1;
            }
            results.insert(key.clone(), tmp_value.clone());
        }
        Ok(results)
    }
}

impl Deref for ResponseValues {
    type Target = HashMap<String, Value>;

    fn deref(&self) -> &HashMap<String, Value> {
        &self.inner
    }
}

impl DerefMut for ResponseValues {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
