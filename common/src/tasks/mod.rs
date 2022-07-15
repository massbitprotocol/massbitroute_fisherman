pub mod command;
pub mod compound;
pub mod dot;
pub mod eth;
pub mod executor;
pub mod http_request;
pub mod ping;
pub mod rpc_request;
pub mod websocket_request;
use crate::job_manage::JobRole;
use log::error;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::fmt::Debug;
use std::fs::metadata;

const DEFAULT_KEY: &str = "default";
/*
 * Load config from a directory of a single file
 */
pub trait LoadConfigs<T: DeserializeOwned + Default + Debug> {
    fn read_configs(config_path: &str, phase: &JobRole) -> Vec<T> {
        let md = metadata(config_path).unwrap_or_else(|err| {
            panic!("Error {:?}. Path not found {}", err, config_path);
        });
        if md.is_dir() {
            Self::read_config_dir(config_path, phase)
        } else {
            Self::read_config_file(config_path, phase)
        }
    }
    /*
    fn read_config_dir(config_path: &str, phase: &JobRole) -> Vec<T> {

    }
    fn read_config_file(path: &str, phase: &JobRole) -> Vec<T> {
        let json_content = std::fs::read_to_string(path).unwrap_or_default();
        let config_value = serde_json::from_str(json_content.as_str()).;
        let configs: Map<String, serde_json::Value> =
            serde_json::from_str(&*json_content).unwrap_or_default();
        let mut task_configs: Vec<T> = Vec::new();
        let default = configs["default"].as_object().unwrap_or_default();
        match configs.get("tasks") {
            None => {

            }
            Some(tasks) => {}
        }
        let tasks = configs["tasks"].as_array().unwrap();
        for config in tasks.iter() {
            let mut map_config = serde_json::Map::from(default.clone());
            let mut task_config = config.as_object().unwrap().clone();
            //log::debug!("Task config before append {:?}", &task_config);
            Self::append(&mut map_config, &mut task_config);
            let value = serde_json::Value::Object(map_config);
            log::trace!("Final task config {:?}", &value);
            match serde_json::from_value::<HttpRequestJobConfig>(value) {
                Ok(config) => {
                    if config.match_phase(phase) {
                        task_configs.push(config)
                    }
                }
                Err(err) => {
                    log::error!("{:?}", &err);
                }
            }
        }
        task_configs
    }
     */
}
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
