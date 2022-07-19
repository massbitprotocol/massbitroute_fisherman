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
use crate::ComponentInfo;
use anyhow::anyhow;
use handlebars::Handlebars;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::metadata;

const DEFAULT_KEY: &str = "default";
const TASKS_KEY: &str = "tasks";
/*
 * Load config from a directory of a single file
 */
pub trait LoadConfigs<T: TaskConfigTrait + DeserializeOwned + Default + Debug> {
    fn read_configs(config_path: &str, phase: &JobRole) -> Vec<T> {
        let md = metadata(config_path).unwrap_or_else(|err| {
            panic!("Error {:?}. Path not found {}", err, config_path);
        });
        if md.is_dir() {
            //Self::read_config_dir(config_path, phase)
            vec![]
        } else {
            let json_content = std::fs::read_to_string(config_path).unwrap_or_default();
            let config_value: Option<Value> = serde_json::from_str(json_content.as_str()).ok();
            if config_value.is_none() {
                return Vec::new();
            }
            let config_value = config_value.unwrap();
            Self::parse_root_value(config_value, phase)
        }
    }
    /*
    fn read_config_dir(config_path: &str, phase: &JobRole) -> Vec<T> {

    }
    */
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
pub trait TaskConfigTrait {
    fn match_phase(&self, phase: &JobRole) -> bool;
    fn match_blockchain(&self, blockchain: &String) -> bool;
    fn match_network(&self, network: &String) -> bool;
    fn match_provider_type(&self, provider_type: &String) -> bool;
    fn can_apply(&self, provider: &ComponentInfo, phase: &JobRole) -> bool;
}
pub trait TemplateRender {
    fn generate_url(
        template: &str,
        handlebars: &Handlebars,
        context: &Value,
    ) -> Result<String, anyhow::Error> {
        // render without register
        handlebars
            .render_template(template, context)
            .map_err(|err| anyhow!("{}", err))
    }
    fn generate_header(
        templates: &Map<String, Value>,
        handlebars: &Handlebars,
        context: &Value,
    ) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        for (key, value) in templates.iter() {
            if let Some(val) = value.as_str() {
                match handlebars.render_template(val, &context) {
                    Ok(header_value) => {
                        headers.insert(key.clone(), header_value);
                    }
                    Err(err) => {
                        log::debug!("Render template error {:?}", &err);
                    }
                }
            } else {
                log::warn!("Value {:?} is not string value", value);
            };
        }
        log::debug!("Generated headers {:?}", &headers);
        headers
    }
    fn generate_body(
        template: &Value,
        handlebars: &Handlebars,
        context: &Value,
    ) -> Result<Value, anyhow::Error> {
        Self::render_template_value(handlebars, template, context)
        //.map(|value| value.to_string())
    }

    fn render_template_value(
        handlebars: &Handlebars,
        value: &Value,
        context: &Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        match value {
            Value::String(val) => {
                let value = handlebars
                    .render_template(val.as_str(), context)
                    .unwrap_or(val.clone());
                Ok(Value::String(value))
            }
            Value::Array(arrs) => {
                let mut vecs = Vec::new();
                for item in arrs.iter() {
                    if let Ok(item_value) = Self::render_template_value(handlebars, item, context) {
                        vecs.push(item_value);
                    }
                }
                Ok(Value::Array(vecs))
            }
            Value::Object(map) => {
                let mut rendered_map: Map<String, Value> = Map::new();
                for (key, item) in map.iter() {
                    if let Ok(item_value) = Self::render_template_value(handlebars, item, context) {
                        rendered_map.insert(key.clone(), item_value);
                    }
                }
                Ok(Value::Object(rendered_map))
            }
            Value::Number(val) => Ok(Value::from(val.clone())),
            Value::Bool(val) => Ok(Value::from(val.clone())),
            Value::Null => Ok(Value::Null),
        }
    }
}
