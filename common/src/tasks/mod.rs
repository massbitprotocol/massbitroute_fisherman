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
use anyhow::{anyhow, Error};
use handlebars::Handlebars;
use log::debug;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::collections::HashMap;
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
            .cloned();

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
        let mut map_config = default.as_ref().cloned().unwrap_or_default();
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
        configs: &[Value],
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
    fn match_blockchain(&self, blockchain: &str) -> bool;
    fn match_network(&self, network: &str) -> bool;
    fn match_provider_type(&self, provider_type: &str) -> bool;
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
                    .unwrap_or_else(|_| val.clone());
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
            Value::Bool(val) => Ok(Value::from(*val)),
            Value::Null => Ok(Value::Null),
        }
    }
}
