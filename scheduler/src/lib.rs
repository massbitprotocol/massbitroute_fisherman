#[macro_use]
extern crate diesel;
extern crate diesel_migrations;

use crate::server_config::Config;
use lazy_static::lazy_static;
use std::env;

pub mod models;
pub mod persistence;
pub mod provider;
pub mod report_processors;
pub mod server_builder;
pub mod server_config;
pub mod service;
pub mod state;
pub mod tasks;

pub const JOB_VERIFICATION_GENERATOR_PERIOD: u64 = 10; //In seconds
pub const JOB_DELIVERY_PERIOD: u64 = 10; //In seconds
pub const JUDGMENT_PERIOD: u64 = 10;
lazy_static! {
    pub static ref COMPONENT_NAME: String = String::from("[Scheduler]");
    pub static ref SCHEDULER_ENDPOINT: String =
        env::var("SCHEDULER_ENDPOINT").unwrap_or(String::from("0.0.0.0:3031"));
    pub static ref REPORT_CALLBACK: String =
        env::var("REPORT_CALLBACK").expect("There is no env var REPORT_CALLBACK");
    pub static ref SCHEDULER_CONFIG: String =
        env::var("SCHEDULER_CONFIG").unwrap_or(String::from("configs/scheduler.json"));
    pub static ref CONNECTION_POOL_SIZE: u32 = env::var("CONNECTION_POOL_SIZE")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(20);
    pub static ref PORTAL_AUTHORIZATION: String =
        env::var("PORTAL_AUTHORIZATION").expect("There is no env var PORTAL_AUTHORIZATION");
    pub static ref DATABASE_URL: String = env::var("DATABASE_URL").unwrap();
    pub static ref REPORT_DIR: String =
        env::var("REPORT_DIR").expect("There is no env var REPORT_DIR");
    pub static ref SIGNER_PHRASE: String =
        env::var("SIGNER_PHRASE").expect("There is no env var SIGNER_PHRASE");
    pub static ref CONFIG_DIR: String =
        env::var("CONFIG_DIR").unwrap_or(String::from("configs/tasks"));
    pub static ref URL_NODES_LIST: String =
        env::var("URL_NODES_LIST").expect("There is no env var URL_NODES_LIST");
    pub static ref URL_GATEWAYS_LIST: String =
        env::var("URL_GATEWAYS_LIST").expect("There is no env var URL_GATEWAYS_LIST");
    pub static ref CONFIG: Config = Config::load(SCHEDULER_CONFIG.as_str());
    pub static ref SQLX_LOGGING: bool =
        env::var("SQLX_LOGGING").ok().and_then(|val|val.parse::<bool>().ok()).unwrap_or(false);
    //time between 2 database flush in ms
     pub static ref LATEST_BLOCK_CACHING_DURATION: i64 =
        env::var("LATEST_BLOCK_CACHING_DURATION").ok().and_then(|val|val.parse::<i64>().ok()).unwrap_or(10000);
    //Worker configurations
    pub static ref WORKER_PATH_JOBS_HANDLE: String =
        env::var("WORKER_PATH_JOBS_HANDLE").unwrap_or(String::from("jobs_handle"));
    pub static ref WORKER_PATH_JOBS_UPDATE: String =
        env::var("WORKER_PATH_JOBS_UPDATE").unwrap_or(String::from("jobs_update"));
    pub static ref WORKER_PATH_JOB_UPDATE: String =
        env::var("WORKER_PATH_GET_STATE").unwrap_or(String::from("get_state"));
}
