use crate::jobs::Job;
use crate::{ComponentInfo, Timestamp};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobHttpRequest {}

impl JobHttpRequest {
    pub fn get_component_url(
        &self,
        config: &HttpRequestJobConfig,
        component: &ComponentInfo,
    ) -> String {
        config.url_template.clone()
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JobHttpResponse {
    pub response_time: Timestamp,
    pub response_body: String,
    pub http_code: u16,
    pub error_code: u32,
    pub message: String,
}

impl JobHttpResponse {
    pub fn new_error(error_code: u32, message: &str) -> Self {
        JobHttpResponse {
            response_time: 0,
            response_body: "".to_string(),
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
    pub headers: serde_json::Map<String, serde_json::Value>,
    pub body: serde_json::Value,
}

/*#[derive(Clone, Serialize, Deserialize, Debug, Defaul)]
struct HttpRequestGeneratorConfig {
    #[serde(with = "serde_with::json::nested")]
    tasks: Vec<HttpRequestJobConfig>,
    #[serde(default)]
    default: HttpRequestJobConfig,
}*/
