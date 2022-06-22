use crate::jobs::Job;
use crate::{ComponentInfo, Timestamp};
use handlebars::{Handlebars, RenderError};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobHttpRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Value>,
    pub response_type: String,
    pub response_values: HashMap<String, Vec<Value>>,
}

impl JobHttpRequest {}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobHttpResponse {
    pub response_time: Timestamp,
    pub detail: JobHttpResponseDetail,
    pub http_code: u16,
    pub error_code: u32,
    pub message: String,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum JobHttpResponseDetail {
    Body(String),
    Values(HashMap<String, Value>),
}

impl Default for JobHttpResponseDetail {
    fn default() -> Self {
        JobHttpResponseDetail::Body(String::new())
    }
}

impl JobHttpResponse {
    pub fn new_error(error_code: u32, message: &str) -> Self {
        JobHttpResponse {
            response_time: 0,
            detail: JobHttpResponseDetail::default(),
            http_code: 0,
            error_code,
            message: message.to_string(),
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobHttpResult {
    pub job: Job,
    pub response: JobHttpResponse,
}

impl JobHttpResult {
    pub fn new(job: Job, response: JobHttpResponse) -> Self {
        JobHttpResult { job, response }
    }
}

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

impl From<HttpRequestError> for JobHttpResponse {
    fn from(error: HttpRequestError) -> Self {
        JobHttpResponse::new_error(error.get_code(), error.get_message().as_str())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct HttpRequestJobConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub request_type: String,
    #[serde(default)]
    pub http_method: String,
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
    pub response: HttpResponseConfig,
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct HttpResponseConfig {
    #[serde(default)]
    pub response_type: String, //Response type: json or text
    #[serde(default)]
    pub values: HashMap<String, Vec<Value>>, //Path to values
}
impl HttpRequestJobConfig {
    pub fn can_apply(&self, provider: &ComponentInfo) -> bool {
        let any = String::from("*");
        let comp_type = provider.component_type.to_string().to_lowercase();
        if !self.provider_types.contains(&any) && !self.provider_types.contains(&comp_type) {
            log::debug!(
                "Component type {:?} not match with {:?}",
                &comp_type,
                &self.provider_types
            );
            return false;
        }
        let blockchain = provider.blockchain.to_lowercase();
        if !self.blockchains.contains(&any) && !self.blockchains.contains(&blockchain) {
            log::debug!(
                "Blockchain {:?} not match with {:?}",
                &provider.blockchain.to_lowercase(),
                &self.blockchains
            );
            return false;
        }
        if !self.networks.contains(&any)
            && !self.networks.contains(&provider.network.to_lowercase())
        {
            log::debug!(
                "Network {:?} not match with {:?}",
                &provider.network.to_lowercase(),
                &self.networks
            );
            return false;
        }
        true
    }
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
            Value::String(val) => Ok(Value::from(val.clone())),
            Value::Number(val) => Ok(Value::from(val.clone())),
            Value::Bool(val) => Ok(Value::from(val.clone())),
            Value::Null => Ok(Value::Null),
        }
    }
}
/*#[derive(Clone, Serialize, Deserialize, Debug, Defaul)]
struct HttpRequestGeneratorConfig {
    #[serde(with = "serde_with::json::nested")]
    tasks: Vec<HttpRequestJobConfig>,
    #[serde(default)]
    default: HttpRequestJobConfig,
}*/
