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
use crate::{BlockChainType, ComponentInfo, NetworkType};
use anyhow::{anyhow, Error};

use log::debug;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use std::fmt::Debug;
use std::fs::metadata;
use std::path::Path;

const DEFAULT_KEY: &str = "default";
const TASKS_KEY: &str = "tasks";
/*
 * Load config from a directory of a single file
 */
pub trait LoadConfigs<T: TaskConfigTrait + DeserializeOwned + Default + Debug> {
    fn read_configs(config_path: &Path, phase: &JobRole) -> Vec<T> {
        let md = metadata(config_path).unwrap_or_else(|err| {
            panic!("Error {:?}. Path not found {:?}", err, config_path);
        });
        if md.is_dir() {
            Self::read_config_dir(config_path, phase)
        } else {
            let configs = std::fs::read_to_string(config_path)
                .ok()
                .and_then(|json_content| {
                    let value: Option<Value> = serde_json::from_str(json_content.as_str()).ok();
                    value.map(|config_value| Self::parse_root_value(config_value, phase))
                });
            configs.unwrap_or_default()
        }
    }

    fn read_config_dir(config_path: &Path, phase: &JobRole) -> Vec<T> {
        //First read default config if exists
        let default_config =
            //std::fs::read_to_string(format!("{}/{}.json", config_path, DEFAULT_KEY))
            std::fs::read_to_string(config_path.join(format!("{DEFAULT_KEY}.json")))
                .map_err(|err| anyhow!("{:?}", &err))
                .and_then(|content| {
                    let config: Result<Map<String, Value>, Error> =
                        serde_json::from_str(content.as_str()).map_err(|err| anyhow!("{:?}", &err));
                    config
                })
                .ok();
        let paths = std::fs::read_dir(config_path).unwrap();
        let mut results = Vec::default();
        for path in paths {
            let config_path = path.unwrap().path();
            debug!("Parse config from path: {}", config_path.display());
            if !config_path.ends_with(format!("{}.json", DEFAULT_KEY)) {
                let configs = std::fs::read_to_string(config_path)
                    .ok()
                    //.map_err(|err| anyhow!("{:?}", &err))
                    .and_then(|json_content| {
                        let config: Option<Value> =
                            serde_json::from_str(json_content.as_str()).ok();
                        //    .map_err(|err| anyhow!("{:?}", &err));
                        config
                    })
                    .and_then(|json_value| Self::parse_value(json_value, &default_config, phase));

                if let Some(mut configs) = configs {
                    results.append(&mut configs);
                }
            }
        }
        results
    }
    fn parse_root_value(config_value: Value, phase: &JobRole) -> Vec<T> {
        let def_config: Option<Map<String, Value>> = config_value
            .get(DEFAULT_KEY)
            .and_then(|value| value.as_object())
            .map(|val| val.clone());

        if config_value.is_object() {
            if let Some(tasks) = config_value.get(TASKS_KEY).and_then(|val| val.as_array()) {
                return Self::parse_array_config(tasks, &def_config, phase);
            } else {
            }
        }
        if config_value.is_array() {
            return Self::parse_array_config(config_value.as_array().unwrap(), &def_config, phase);
        }
        Vec::new()
    }
    fn parse_value(
        config_value: Value,
        default_config: &Option<Map<String, Value>>,
        phase: &JobRole,
    ) -> Option<Vec<T>> {
        if config_value.is_object() {
            Self::parse_object_config(config_value.as_object().unwrap(), default_config, phase)
                .ok()
                .map(|config| vec![config])
        } else if config_value.is_array() {
            Some(Self::parse_array_config(
                config_value.as_array().unwrap(),
                default_config,
                phase,
            ))
        } else {
            None
        }
    }
    fn parse_object_config(
        config: &Map<String, Value>,
        default: &Option<Map<String, Value>>,
        phase: &JobRole,
    ) -> Result<T, anyhow::Error> {
        let mut map_config = default.as_ref().map(|val| val.clone()).unwrap_or_default();
        //log::debug!("Task config before append {:?}", &task_config);
        Self::append(&mut map_config, config);
        let value = serde_json::Value::Object(map_config);
        log::trace!("Final task config {:?}", &value);
        match serde_json::from_value::<T>(value) {
            Ok(config) => {
                if config.match_phase(phase) {
                    Ok(config)
                } else {
                    Err(anyhow!("Phase not match"))
                }
            }
            Err(err) => Err(anyhow!("{:?}", &err)),
        }
    }
    fn parse_array_config(
        configs: &Vec<Value>,
        default: &Option<Map<String, Value>>,
        phase: &JobRole,
    ) -> Vec<T> {
        let mut result = Vec::new();
        for config in configs.iter() {
            if let Some(config_value) = config.as_object() {
                if let Ok(config) = Self::parse_object_config(config_value, default, phase) {
                    result.push(config);
                }
            }
        }
        result
    }
    //Todo: Implement Deep append
    fn append(target: &mut Map<String, Value>, source: &Map<String, Value>) {
        target.append(&mut source.clone());
    }
}

pub trait TaskConfigTrait {
    fn match_phase(&self, phase: &JobRole) -> bool;
    fn get_blockchain(&self) -> &Vec<String>;
    fn match_blockchain(&self, blockchain: &BlockChainType) -> bool {
        let blockchain = blockchain.to_string().to_lowercase();
        if !self.get_blockchain().contains(&String::from("*"))
            && !self.get_blockchain().contains(&blockchain)
        {
            log::trace!(
                "Blockchain {:?} not match with {:?}",
                &blockchain,
                &self.get_blockchain()
            );
            return false;
        }
        true
    }
    fn match_network(&self, network: &NetworkType) -> bool;
    fn match_provider_type(&self, provider_type: &String) -> bool;
    fn can_apply(&self, provider: &ComponentInfo, phase: &JobRole) -> bool;
}
