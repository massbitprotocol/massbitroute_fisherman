pub mod command;
pub mod compound;
pub mod dot;
pub mod eth;
pub mod executor;
pub mod http_request;
pub mod ping;
pub mod rpc_request;

use crate::job_manage::JobRole;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::fmt::Debug;

const DEFAULT_KEY: &str = "default";

pub trait LoadConfig<T: DeserializeOwned + Default + Debug> {
    fn load_config(path: &str, role: &JobRole) -> T {
        let json = std::fs::read_to_string(path).unwrap_or_else(|err| {
            panic!("Error {:?}. Path not found {}", err, path);
        });
        let configs: Value = serde_json::from_str(&*json).unwrap_or_else(|err| {
            panic!("{:?}", &err);
        });

        let phase_config_value = if let Some(phase_config) = configs.get(role.to_string()) {
            if let Some(default_config) = configs.get(DEFAULT_KEY) {
                let mut phase_object = phase_config.as_object().unwrap().clone();
                Self::append(&mut phase_object, default_config.as_object().unwrap());
                serde_json::Value::Object(phase_object)
            } else {
                phase_config.clone()
            }
        } else if let Some(default_config) = configs.get(DEFAULT_KEY) {
            default_config.clone()
        } else {
            panic!("Please check default or {} config", role.to_string());
        };
        log::info!("Final phase config {:?}", phase_config_value);
        match serde_json::from_value(phase_config_value) {
            Ok(config) => config,
            Err(err) => {
                panic!("{:?}", &err);
            }
        }
    }
    //Todo: Implement Deep append
    fn append(target: &mut Map<String, Value>, source: &Map<String, Value>) {
        for (key, value) in source {
            target.insert(key.clone(), value.clone());
        }
    }
    /*
    fn load_config(path: &str, role: &JobRole) -> T {
        let json = std::fs::read_to_string(path).unwrap_or_else(|err| {
            panic!("Error {:?}. Path not found {}", err, path);
        });
        let configs: Value = serde_json::from_str(&*json).unwrap();
        let mut default_config: T =
            serde_json::from_value(configs.get(DEFAULT_KEY).unwrap().clone()).unwrap();
        if let Some(modify_config) = configs.get(role.to_string()) {
            let modify_config: Result<T, _> = serde_json::from_value(modify_config.clone());
            if let Ok(modify_config) = modify_config {
                default_config = modify_config;
            }
        }
        info!("Loaded config: {:#?} for role: {:?}", default_config, role);
        default_config
    }
     */
}
