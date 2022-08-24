pub mod component;
pub mod job_action;
pub mod job_manage;
pub mod jobs;
pub use logger;
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
