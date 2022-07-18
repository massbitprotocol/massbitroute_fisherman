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
use handlebars::Handlebars;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::collections::HashMap;
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

pub trait TemplateRender {
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
