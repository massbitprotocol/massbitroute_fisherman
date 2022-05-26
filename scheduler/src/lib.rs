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
        env::var("SCHEDULER_ENDPOINT").unwrap_or(String::from("0.0.0.0:3030"));
}