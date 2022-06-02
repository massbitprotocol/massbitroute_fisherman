#[macro_use]
extern crate diesel;
extern crate diesel_migrations;

use dotenv;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::env;

pub mod models;
pub mod provider;
pub mod seaorm;
pub mod server_builder;
pub mod server_config;
pub mod service;
pub mod state;
pub const JOB_GENERATOR_PERIOD: u64 = 10; //In seconds
pub const JOB_DELIVERY_PERIOD: u64 = 10; //In seconds
lazy_static! {
    pub static ref COMPONENT_NAME: String = String::from("[Scheduler]");
    pub static ref SCHEDULER_ENDPOINT: String =
        env::var("SCHEDULER_ENDPOINT").unwrap_or(String::from("0.0.0.0:3031"));
    pub static ref SCHEDULER_CONFIG: String =
        env::var("SCHEDULER_CONFIG").unwrap_or(String::from("config_fisherman.json"));
    pub static ref CONNECTION_POOL_SIZE: u32 = env::var("CONNECTION_POOL_SIZE")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(20);
    pub static ref PORTAL_AUTHORIZATION: String =
        env::var("PORTAL_AUTHORIZATION").expect("There is no env var PORTAL_AUTHORIZATION");
    pub static ref DATABASE_URL: String = env::var("DATABASE_URL").unwrap_or(String::from(
        "postgres://fisherman:FishermanCodelight123@35.193.163.173:5432/massbit-fisherman"
    ));
}
