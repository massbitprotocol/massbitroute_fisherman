pub mod server_builder;
pub mod server_config;
pub mod service;
pub mod state;
use dotenv;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::env;

lazy_static! {
    pub static ref SCHEDULER_ENDPOINT: String = env::var("SCHEDULER_ENDPOINT")
        .unwrap_or(String::from("https://scheduler.massbitroute.net"));
    pub static ref WORKER_ID: String =
        env::var("WORKER_ID").unwrap_or(String::from("someworkerhash"));
    pub static ref ZONE: String = env::var("ZONE")
        .unwrap_or(String::from("AS"))
        .to_uppercase();
    pub static ref ENVIRONMENT: String = env::var("ENVIRONMENT").unwrap_or(String::from("local"));
    pub static ref FISHERMAN_ENDPOINT: String =
        env::var("FISHERMAN_ENDPOINT").unwrap_or(String::from("0.0.0.0:4040"));
    pub static ref FISHERMAN_MAX_PARALLEL_JOBS: usize = env::var("MAX_PARALLEL_JOBS").unwrap_or(8);
}
