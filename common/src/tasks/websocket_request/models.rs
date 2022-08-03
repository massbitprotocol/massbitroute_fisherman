use crate::component::ChainInfo;
use crate::job_manage::JobRole;
use crate::jobs::AssignmentConfig;
use crate::models::{ResponseConfig, ResponseValues};
use crate::tasks::{LoadConfigs, TaskConfigTrait};
use crate::{ComponentInfo, Timestamp};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct JobWebsocket {
    pub url: String,
    pub chain_info: Option<ChainInfo>,
    pub headers: HashMap<String, String>,
    pub body: Option<Value>,
    pub response_type: String,
    pub response_values: HashMap<String, Vec<Value>>,
}

impl JobWebsocket {}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobWebsocketResponse {
    pub request_timestamp: Timestamp, //Time to call request in second
    pub response_duration: Timestamp, //Time to receipt response;
    pub detail: JobWebsocketResponseDetail,
    pub error_code: u32,
    pub message: String,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum JobWebsocketResponseDetail {
    Body(String),
    Values(ResponseValues),
}

impl Default for JobWebsocketResponseDetail {
    fn default() -> Self {
        JobWebsocketResponseDetail::Body(String::new())
    }
}

impl JobWebsocketResponse {
    pub fn new_error(request_time: Timestamp, error_code: u32, message: &str) -> Self {
        JobWebsocketResponse {
            request_timestamp: request_time,
            response_duration: request_time,
            detail: JobWebsocketResponseDetail::default(),
            error_code,
            message: message.to_string(),
        }
    }
}
pub type JobWebsocketResult = JobWebsocketResponse;

#[derive(Error, Debug, Clone)]
pub enum HttpRequestError {
    #[error("build error")]
    BuildError(String),
    #[error("send error")]
    SendError(String),
    #[error("get body error")]
    GetBodyError(String),
}

impl HttpRequestError {
    pub fn get_message(&self) -> String {
        match self {
            HttpRequestError::BuildError(message)
            | HttpRequestError::SendError(message)
            | HttpRequestError::GetBodyError(message) => message.to_string(),
        }
    }
    pub fn get_code(&self) -> u32 {
        match self {
            HttpRequestError::BuildError(_) => 1u32,
            HttpRequestError::SendError(_) => 2u32,
            HttpRequestError::GetBodyError(_) => 3u32,
        }
    }
}

// impl From<HttpRequestError> for JobHttpResponse {
//     fn from(error: HttpRequestError) -> Self {
//         JobHttpResponse::new_error(error.get_code(), error.get_message().as_str())
//     }
// }

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobWebsocketConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub request_type: String,
    #[serde(default)]
    pub phases: Vec<String>,
    #[serde(default)]
    pub url_template: String,
    #[serde(default)]
    pub request_timeout: Timestamp,
    pub repeat_number: i32,
    #[serde(default)]
    pub provider_types: Vec<String>,
    #[serde(default)]
    pub blockchains: Vec<String>,
    #[serde(default)]
    pub networks: Vec<String>,
    pub headers: serde_json::Map<String, serde_json::Value>,
    pub body: serde_json::Value,
    pub response: ResponseConfig,
    pub assignment: AssignmentConfig,
    pub interval: Timestamp,
    #[serde(default)]
    pub thresholds: serde_json::Map<String, serde_json::Value>,
}

impl fmt::Display for JobWebsocketConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({} {} {})",
            self.name, self.request_type, self.repeat_number,
        )
    }
}

impl LoadConfigs<JobWebsocketConfig> for JobWebsocketConfig {}
impl JobWebsocketConfig {
    // pub fn read_config(path: &str, phase: &JobRole) -> Vec<JobWebsocketConfig> {
    //     let json_content = std::fs::read_to_string(path).unwrap_or_default();
    //     let configs: Map<String, serde_json::Value> =
    //         serde_json::from_str(&*json_content).unwrap_or_default();
    //     let mut task_configs: Vec<JobWebsocketConfig> = Vec::new();
    //     let default = configs["default"].as_object().unwrap();
    //     let tasks = configs["tasks"].as_array().unwrap();
    //     for config in tasks.iter() {
    //         let mut map_config = serde_json::Map::from(default.clone());
    //         let mut task_config = config.as_object().unwrap().clone();
    //         //log::debug!("Task config before append {:?}", &task_config);
    //         Self::append(&mut map_config, &mut task_config);
    //         let value = serde_json::Value::Object(map_config);
    //         log::trace!("Final task config {:?}", &value);
    //         match serde_json::from_value::<JobWebsocketConfig>(value) {
    //             Ok(config) => {
    //                 if config.match_phase(phase) {
    //                     task_configs.push(config)
    //                 }
    //             }
    //             Err(err) => {
    //                 log::error!("{:?}", &err);
    //             }
    //         }
    //     }
    //     task_configs
    // }
    //Todo: Implement Deep append
    pub fn append(target: &mut Map<String, Value>, source: &mut Map<String, Value>) {
        target.append(source);
    }
}

impl TaskConfigTrait for JobWebsocketConfig {
    fn match_phase(&self, phase: &JobRole) -> bool {
        self.phases.contains(&String::from("*")) || self.phases.contains(&phase.to_string())
    }
    fn match_blockchain(&self, blockchain: &String) -> bool {
        let blockchain = blockchain.to_lowercase();
        if !self.blockchains.contains(&String::from("*")) && !self.blockchains.contains(&blockchain)
        {
            log::trace!(
                "Blockchain {:?} not match with {:?}",
                &blockchain,
                &self.blockchains
            );
            return false;
        }
        true
    }
    fn match_network(&self, network: &String) -> bool {
        let network = network.to_lowercase();
        if !self.networks.contains(&String::from("*")) && !self.networks.contains(&network) {
            log::trace!(
                "Networks {:?} not match with {:?}",
                &network,
                &self.networks
            );
            return false;
        }
        true
    }
    fn match_provider_type(&self, provider_type: &String) -> bool {
        let provider_type = provider_type.to_lowercase();
        if !self.provider_types.contains(&String::from("*"))
            && !self.provider_types.contains(&provider_type)
        {
            log::trace!(
                "Provider type {:?} not match with {:?}",
                &provider_type,
                &self.networks
            );
            return false;
        }
        true
    }
    fn can_apply(&self, provider: &ComponentInfo, phase: &JobRole) -> bool {
        if !self.active {
            return false;
        }
        // Check phase
        if !self.match_phase(phase) {
            return false;
        }
        if !self.match_provider_type(&provider.component_type.to_string()) {
            return false;
        }
        if !self.match_blockchain(&provider.blockchain) {
            return false;
        }
        if !self.match_network(&provider.network) {
            return false;
        }
        true
    }
}

impl JobWebsocketConfig {
    pub fn generate_header(
        &self,
        handlebars: &Handlebars,
        context: &Value,
    ) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        for (key, value) in self.headers.iter() {
            if let Some(val) = value.as_str() {
                match handlebars.render_template(val, &context) {
                    Ok(header_value) => {
                        headers.insert(key.clone(), header_value);
                    }
                    Err(err) => {
                        log::debug!("Render template error {:?}", &err);
                    }
                }
            } else {
                log::warn!("Value {:?} is not string value", value);
            };
        }
        log::debug!("Generated headers {:?}", &headers);
        headers
    }
    pub fn generate_body(
        &self,
        handlebars: &Handlebars,
        context: &Value,
    ) -> Result<Value, anyhow::Error> {
        let body = self.body.clone();
        self.render_template_value(handlebars, &body, context)
        //.map(|value| value.to_string())
    }

    pub fn render_template_value(
        &self,
        handlebars: &Handlebars,
        value: &Value,
        context: &Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        match value {
            Value::String(val) => {
                let value = handlebars
                    .render_template(val.as_str(), context)
                    .unwrap_or(val.clone());
                Ok(Value::String(value))
            }
            Value::Array(arrs) => {
                let mut vecs = Vec::new();
                for item in arrs.iter() {
                    if let Ok(item_value) = self.render_template_value(handlebars, item, context) {
                        vecs.push(item_value);
                    }
                }
                Ok(Value::Array(vecs))
            }
            Value::Object(map) => {
                let mut rendered_map: Map<String, Value> = Map::new();
                for (key, item) in map.iter() {
                    if let Ok(item_value) = self.render_template_value(handlebars, item, context) {
                        rendered_map.insert(key.clone(), item_value);
                    }
                }
                Ok(Value::Object(rendered_map))
            }
            Value::Number(val) => Ok(Value::from(val.clone())),
            Value::Bool(val) => Ok(Value::from(val.clone())),
            Value::Null => Ok(Value::Null),
        }
    }
}
