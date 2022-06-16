pub mod command;
pub mod compound;
pub mod dot;
pub mod eth;
pub mod executor;
pub mod generator;
pub mod http_request;
pub mod ping;
pub mod rpc_request;
use crate::job_manage::JobRole;
use crate::Timestamp;
pub use executor::get_executors;
use log::{error, info};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::iter::Map;

const DEFAULT_KEY: &str = "default";
const VERIFICATION_KEY: &str = "verification";
const FISHERMAN_KEY: &str = "fisherman";

pub trait LoadConfig<T: DeserializeOwned + Default + Debug> {
    fn load_config(path: &str, role: &JobRole) -> T {
        let json = std::fs::read_to_string(path).unwrap_or_else(|err| {
            panic!("Error {:?}. Path not found {}", err, path);
        });
        let configs: Value = serde_json::from_str(&*json).unwrap();
        let mut default_config: T =
            serde_json::from_value(configs.get(DEFAULT_KEY).unwrap().clone()).unwrap();
        if let Some(modify_config) = configs.get(role.to_string()) {
            let mut modify_config: Result<T, _> = serde_json::from_value(modify_config.clone());
            if let Ok(modify_config) = modify_config {
                default_config = modify_config;
            }
        }
        info!("Loaded config: {:#?} for role: {:?}", default_config, role);
        default_config
    }
}
