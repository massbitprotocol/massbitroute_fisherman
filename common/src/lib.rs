pub mod component;
pub mod job_action;
pub mod job_manage;
pub mod jobs;

use anyhow::anyhow;
pub use logger;
use std::str::FromStr;

pub mod models;
pub mod task_spawn;
pub mod tasks;
pub mod types;
pub mod util;
pub mod workers;

pub use crate::component::ComponentInfo;
use lazy_static::lazy_static;
use log::error;
pub use types::*;
const DEFAULT_JOB_INTERVAL: Timestamp = 1000;
const DEFAULT_JOB_TIMEOUT: Timestamp = 5000;
lazy_static! {
    pub static ref DOMAIN: String = env::var("DOMAIN").expect("There is no env var DOMAIN");
    pub static ref WORKER_ID: String =
        env::var("WORKER_ID").expect("There is no env var WORKER_ID");
    pub static ref COMMON_CONFIG_FILE: String =
        env::var("COMMON_CONFIG_FILE").expect("There is no env var COMMON_CONFIG_FILE");
    pub static ref COMMON_CONFIG: Config = Config::load(COMMON_CONFIG_FILE.as_str());
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub default_http_request_timeout_ms: u64, //time in ms
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_http_request_timeout_ms: 5000,
        }
    }
}

impl Config {
    pub fn load(file_path: &str) -> Self {
        let json = std::fs::read_to_string(file_path).unwrap_or_else(|err| {
            error!(
                "Unable to read config file for common`{}`: {}",
                file_path, err
            );
            String::default()
        });
        let config: Config = serde_json::from_str(&*json).unwrap_or_default();
        config
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Environment {
    Local,
    DockerTest,
    Release,
    Production,
}

impl ToString for Environment {
    fn to_string(&self) -> String {
        match self {
            Environment::Local => "local".to_string(),
            Environment::DockerTest => "docker_test".to_string(),
            Environment::Release => "release".to_string(),
            Environment::Production => "production".to_string(),
        }
    }
}

impl FromStr for Environment {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(Environment::Local),
            "docker_test" => Ok(Environment::DockerTest),
            "production" => Ok(Environment::Production),
            _ => Err(anyhow!("Cannot parse {s} to Environment")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Scheme {
    Https,
    Http,
}

impl Scheme {
    pub fn to_http_string(&self) -> String {
        match self {
            Scheme::Https => "https".to_string(),
            Scheme::Http => "http".to_string(),
        }
    }
    pub fn to_ws_string(&self) -> String {
        match self {
            Scheme::Https => "wss".to_string(),
            Scheme::Http => "ws".to_string(),
        }
    }
}

impl FromStr for Scheme {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "https" => Ok(Scheme::Https),
            "http" => Ok(Scheme::Http),
            _ => Err(anyhow!("Cannot parse {s} to Schema")),
        }
    }
}
