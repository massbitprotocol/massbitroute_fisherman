pub mod server_builder;
pub mod server_config;
pub mod service;
pub mod state;
use lazy_static::lazy_static;
use dotenv;
use serde::Deserialize;
use std::env;

lazy_static! {
    pub static ref SCHEDULER_ENDPOINT: String =
        env::var("SCHEDULER_ENDPOINT").unwrap_or(String::from("https://scheduler.massbitroute.net"));
     pub static ref WORKER_ID: String =
        env::var("WORKER_ID").unwrap_or(String::from("someworkerhash"));
    pub static ref FISHERMAN_ENDPOINT: String =
        env::var("FISHERMAN_ENDPOINT").unwrap_or(String::from("0.0.0.0:4040"));
}