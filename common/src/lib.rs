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
pub use types::*;
const DEFAULT_JOB_INTERVAL: Timestamp = 1000;
const DEFAULT_JOB_TIMEOUT: Timestamp = 5000;
lazy_static! {
    pub static ref DOMAIN: String = env::var("DOMAIN").expect("There is no env var DOMAIN");
    pub static ref WORKER_ID: String =
        env::var("WORKER_ID").expect("There is no env var WORKER_ID");
    pub static ref COMMON_CONFIG_FILE: String =
        env::var("COMMON_CONFIG_FILE").unwrap_or_else(|_| String::from("common.json"));
    pub static ref COMMON_CONFIG: Config = Config::load(COMMON_CONFIG_FILE.as_str());
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub default_http_request_timeout_ms: u64, //time in ms
}

impl Config {
    pub fn load(file_path: &str) -> Self {
        let json = std::fs::read_to_string(file_path).unwrap_or_else(|err| {
            panic!(
                "Unable to read config file for common`{}`: {}",
                file_path, err
            )
        });
        let config: Config = serde_json::from_str(&*json).unwrap();
        config
    }
}
