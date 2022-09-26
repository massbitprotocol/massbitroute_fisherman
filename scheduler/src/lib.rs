extern crate diesel;
extern crate diesel_migrations;

use common::Scheme;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::env;

pub mod handler;
pub mod models;
pub mod persistence;
pub mod provider;
pub mod report_processors;
pub mod server_builder;
pub mod server_config;
pub mod service;
pub mod state;
pub mod tasks;

use crate::tasks::benchmark::generator::BenchmarkConfig;
use anyhow::anyhow;
use common::tasks::http_request::HttpRequestJobConfig;
use common::tasks::websocket_request::JobWebsocketConfig;
use handlebars::Handlebars;
use serde_json::{Map, Value};
use server_config::Config;
use std::str::FromStr;

pub const JOB_VERIFICATION_GENERATOR_PERIOD: u64 = 10; //In seconds
pub const DELIVERY_PERIOD: u64 = 10; //In seconds
pub const JUDGMENT_PERIOD: u64 = 10;
pub const RESULT_CACHE_MAX_LENGTH: usize = 10;

lazy_static! {
    pub static ref COMPONENT_NAME: String = String::from("[Scheduler]");
    pub static ref SCHEDULER_ENDPOINT: String =
        env::var("SCHEDULER_ENDPOINT").unwrap_or_else(|_| String::from("0.0.0.0:3031"));
    pub static ref REPORT_CALLBACK: String =
        env::var("REPORT_CALLBACK").expect("There is no env var REPORT_CALLBACK");
    pub static ref CONFIG_DIR: String =
        env::var("CONFIG_DIR").unwrap_or_else(|_| String::from("configs/"));
    pub static ref SCHEDULER_CONFIG: String = format!("{}/scheduler.json",&*CONFIG_DIR);
        //env::var("SCHEDULER_CONFIG").unwrap_or_else(|_| String::from("configs/scheduler.json"));
    pub static ref CONNECTION_POOL_SIZE: u32 = env::var("CONNECTION_POOL_SIZE")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(20);
    pub static ref DATABASE_URL: String = env::var("DATABASE_URL").unwrap();
    pub static ref REPORT_DIR: String =
        env::var("REPORT_DIR").expect("There is no env var REPORT_DIR");
    pub static ref SIGNER_PHRASE: String =
        env::var("SIGNER_PHRASE").expect("There is no env var SIGNER_PHRASE");
    pub static ref CONFIG_TASK_DIR: String = format!("{}/tasks",&*CONFIG_DIR);
    pub static ref LOG_CONFIG: String = format!("{}/log.yaml",&*CONFIG_DIR);
        //env::var("CONFIG_TASK_DIR").unwrap_or_else(|_| String::from("configs/tasks"));
    pub static ref CONFIG_HTTP_REQUEST_DIR: String = String::from("http_request");
    pub static ref CONFIG_BENCHMARK_DIR: String = String::from("benchmark");
    pub static ref CONFIG_WEBSOCKET_DIR: String = String::from("websocket");
    pub static ref PORTAL_AUTHORIZATION: String =
        env::var("PORTAL_AUTHORIZATION").expect("There is no env var PORTAL_AUTHORIZATION");
    pub static ref SCHEDULER_AUTHORIZATION: String =
        env::var("SCHEDULER_AUTHORIZATION").expect("There is no env var SCHEDULER_AUTHORIZATION");
    pub static ref URL_PORTAL: String =
        env::var("URL_PORTAL").expect("There is no env var URL_PORTAL, e.g. https://portal.massbitroute.net");
    pub static ref URL_CHAIN: String =
        env::var("URL_CHAIN").unwrap_or_else(|_| "ws://chain.massbitroute.net:9944".to_string());
    pub static ref URL_NODES_LIST: String = format!("{}/{}",*URL_PORTAL,
        env::var("PATH_NODES_LIST").expect("There is no env var PATH_NODES_LIST, e.g. mbr/node/list/verify"));
    pub static ref URL_GATEWAYS_LIST: String = format!("{}/{}",*URL_PORTAL,
        env::var("PATH_GATEWAYS_LIST").expect("There is no env var URL_GATEWAYS_LIST, e.g. mbr/gateway/list/verify"));
    pub static ref URL_PORTAL_PROVIDER_REPORT: String = format!("{}/{}",*URL_PORTAL,
        env::var("PATH_PORTAL_PROVIDER_REPORT").expect("There is no env var PATH_PORTAL_PROVIDER_REPORT, e.g. mbr/benchmark"));
    pub static ref URL_PORTAL_PROVIDER_VERIFY: String = format!("{}/{}",*URL_PORTAL,
        env::var("PATH_PORTAL_PROVIDER_VERIFY").expect("There is no env var PATH_PORTAL_PROVIDER_VERIFY, e.g. mbr/verify"));
    pub static ref CONFIG: Config = Config::load(SCHEDULER_CONFIG.as_str());
    pub static ref SQLX_LOGGING: bool =
        env::var("SQLX_LOGGING").ok().and_then(|val|val.parse::<bool>().ok()).unwrap_or(false);
    //time between 2 database flush in ms
     pub static ref LATEST_BLOCK_CACHING_DURATION: i64 =
        env::var("LATEST_BLOCK_CACHING_DURATION").ok().and_then(|val|val.parse::<i64>().ok()).unwrap_or(10000);
    //Worker configurations
    pub static ref WORKER_PATH_JOBS_HANDLE: String =
        env::var("WORKER_PATH_JOBS_HANDLE").unwrap_or_else(|_| String::from("handle_jobs"));
    pub static ref WORKER_PATH_JOBS_UPDATE: String =
        env::var("WORKER_PATH_JOBS_UPDATE").unwrap_or_else(|_| String::from("jobs_update"));
    pub static ref WORKER_PATH_JOB_UPDATE: String =
        env::var("WORKER_PATH_GET_STATE").unwrap_or_else(|_| String::from("get_state"));
    pub static ref IS_VERIFY_REPORT: bool =
        env::var("IS_VERIFY_REPORT").ok().and_then(|val|val.parse::<bool>().ok()).expect("There is no env var IS_VERIFY_REPORT, e.g. true");
    pub static ref IS_REGULAR_REPORT: bool =
        env::var("IS_REGULAR_REPORT").ok().and_then(|val|val.parse::<bool>().ok()).expect("There is no env var IS_REGULAR_REPORT, e.g. true");
    pub static ref BUILD_VERSION: String = format!("{}", env!("BUILD_VERSION"));
    // pub static ref ENVIRONMENT: Environment = Environment::from_str(
    //     &env::var("ENVIRONMENT").expect("There is no env var ENVIRONMENT")).expect("Cannot parse var ENVIRONMENT");
    pub static ref SCHEME: Scheme = Scheme::from_str(
        &env::var("SCHEME").expect("There is no env var SCHEME")).expect("Cannot parse var SCHEME");
    pub static ref IS_REGULAR_WORKER_ONCHAIN: bool =
        env::var("IS_REGULAR_WORKER_ONCHAIN").ok().and_then(|val|val.parse::<bool>().ok()).unwrap_or(true);

}

pub trait TemplateRender {
    fn generate_url(
        template: &str,
        handlebars: &Handlebars,
        context: &Value,
    ) -> Result<String, anyhow::Error> {
        // render without register
        handlebars
            .render_template(&template, context)
            .map_err(|err| anyhow!("{}", err))
    }
    fn generate_header(
        templates: &Map<String, Value>,
        handlebars: &Handlebars,
        context: &Value,
    ) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        for (key, value) in templates.iter() {
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
    fn generate_body(
        template: &Value,
        handlebars: &Handlebars,
        context: &Value,
    ) -> Result<Value, anyhow::Error> {
        Self::render_template_value(handlebars, template, context)
        //.map(|value| value.to_string())
    }

    fn render_template_value(
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
                    if let Ok(item_value) = Self::render_template_value(handlebars, item, context) {
                        vecs.push(item_value);
                    }
                }
                Ok(Value::Array(vecs))
            }
            Value::Object(map) => {
                let mut rendered_map: Map<String, Value> = Map::new();
                for (key, item) in map.iter() {
                    if let Ok(item_value) = Self::render_template_value(handlebars, item, context) {
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
impl TemplateRender for JobWebsocketConfig {}
impl TemplateRender for HttpRequestJobConfig {}
impl TemplateRender for BenchmarkConfig {}
