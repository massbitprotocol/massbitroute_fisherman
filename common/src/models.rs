use reqwest::Body;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WorkerInfo {
    worker_id: String,
    url: String,
    worker_spec: String,
}

impl WorkerInfo {
    pub fn new() -> Self {
        WorkerInfo {
            worker_id: "".to_string(),
            url: "".to_string(),
            worker_spec: "".to_string(),
        }
    }
}

impl Into<Body> for WorkerInfo {
    fn into(self) -> Body {
        todo!()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WorkerRegisterResult {
    worker_id: String,
    result: String,
}

impl WorkerRegisterResult {
    pub fn new() -> Self {
        WorkerRegisterResult {
            worker_id: "".to_string(),
            result: "".to_string(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WorkerStateParam {}
