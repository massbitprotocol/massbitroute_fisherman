pub mod dot;
pub mod eth;
pub mod executor;
pub mod generator;
pub mod ping;

use crate::job_manage::JobRole;
pub use executor::get_eth_executors;
use log::error;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::iter::Map;

const DEFAULT_KEY: &str = "default";
const VERIFICATION_KEY: &str = "verification";
const FISHERMAN_KEY: &str = "fisherman";
pub trait LoadConfig<T: DeserializeOwned + Default> {
    fn load_config(path: &str, role: &JobRole) -> T {
        let json = std::fs::read_to_string(path).unwrap();
        let configs: Value = serde_json::from_str(&*json).unwrap();
        let mut default_config: T =
            serde_json::from_value(configs.get(DEFAULT_KEY).unwrap().clone()).unwrap();
        let mut modify_config: Result<T, _> =
            serde_json::from_value(configs.get(role.to_string()).unwrap().clone());
        if let Ok(modify_config) = modify_config {
            default_config = modify_config;
        }
        default_config
    }
}
