use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseValues {
    inner: HashMap<String, Value>,
}

impl ResponseValues {
    pub fn new(inner: HashMap<String, Value>) -> Self {
        ResponseValues { inner }
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
