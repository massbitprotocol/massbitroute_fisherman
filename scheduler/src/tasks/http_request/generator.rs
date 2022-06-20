use crate::tasks::generator::TaskApplicant;
use anyhow::{Context, Error};
use async_trait::async_trait;
use common::component::ComponentInfo;
use common::job_manage::JobDetail;
use common::jobs::Job;
use common::tasks::http_request::{HttpRequestJobConfig, JobHttpRequest};
use common::PlanId;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::sync::Arc;
use std::vec;
use tokio::sync::mpsc::Sender;

/*
 * Periodically ping to node/gateway to get response time, to make sure node/gateway is working
 */
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct HttpRequestGenerator {
    root_config: serde_json::Map<String, serde_json::Value>,
    task_configs: Vec<HttpRequestJobConfig>,
}

impl HttpRequestGenerator {
    pub fn get_name() -> String {
        String::from("HttpRequest")
    }
    pub fn new(config_dir: &str) -> Result<Self, anyhow::Error> {
        let path = format!("{}/rpcrequest.json", config_dir);
        let json_content = std::fs::read_to_string(path.as_str())?;
        let mut configs: Map<String, serde_json::Value> = serde_json::from_str(&*json_content)?;
        let mut task_configs = Vec::new();
        let default = configs["default"].as_object().unwrap();
        let mut tasks = configs["tasks"].as_array().unwrap();
        for config in tasks.iter() {
            let mut map_config = serde_json::Map::from(default.clone());
            let mut task_config = config.as_object().unwrap().clone();
            map_config.append(&mut task_config);
            let value = serde_json::Value::Object(map_config);
            println!("{:?}", &value);
            let http_request_config: HttpRequestJobConfig = serde_json::from_value(value)?;
            task_configs.push(http_request_config);
        }
        Ok(HttpRequestGenerator {
            root_config: configs,
            task_configs,
        })
    }
    pub fn get_url(&self, component: &ComponentInfo) -> String {
        format!("https://{}/_ping", component.ip)
    }
}

impl TaskApplicant for HttpRequestGenerator {
    fn can_apply(&self, component: &ComponentInfo) -> bool {
        true
    }

    fn apply(&self, plan_id: &PlanId, component: &ComponentInfo) -> Result<Vec<Job>, Error> {
        log::debug!("Http Request apply for component {:?}", component);
        let mut jobs = Vec::new();
        for config in self.task_configs.iter() {
            let detail = JobHttpRequest {};
            let comp_url = detail.get_component_url(config, component);
            let mut job = Job::new(plan_id.clone(), component, JobDetail::HttpRequest(detail));
            job.parallelable = true;
            job.component_url = comp_url;
            job.timeout = config.request_timeout;
            job.repeat_number = config.repeat_number;
            jobs.push(job);
        }
        Ok(jobs)
    }
}
