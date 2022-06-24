use crate::persistence::PlanModel;
use crate::tasks::generator::TaskApplicant;
use anyhow::{anyhow, Context, Error};
use async_trait::async_trait;
use common::component::{ComponentInfo, ComponentType};
use common::job_manage::{JobDetail, JobRole};
use common::jobs::{Job, JobAssignment};
use common::tasks::http_request::{HttpRequestJobConfig, JobHttpRequest};
use common::workers::MatchedWorkers;
use common::{PlanId, DOMAIN};
use handlebars::Handlebars;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::vec;
use tokio::sync::mpsc::Sender;
use warp::head;
use warp::reply::json;

/*
 * Periodically ping to node/gateway to get response time, to make sure node/gateway is working
 */
#[derive(Clone, Debug, Default)]
pub struct HttpRequestGenerator {
    root_config: serde_json::Map<String, serde_json::Value>,
    task_configs: Vec<HttpRequestJobConfig>,
    handlebars: Handlebars<'static>,
}

impl HttpRequestGenerator {
    pub fn get_name() -> String {
        String::from("HttpRequest")
    }
    pub fn new(config_dir: &str) -> Self {
        let path = format!("{}/http_request.json", config_dir);
        let json_content = std::fs::read_to_string(path.as_str()).unwrap_or_default();
        let mut configs: Map<String, serde_json::Value> =
            serde_json::from_str(&*json_content).unwrap_or_default();
        let mut task_configs = Vec::new();
        let default = configs["default"].as_object().unwrap();
        let mut tasks = configs["tasks"].as_array().unwrap();
        for config in tasks.iter() {
            let mut map_config = serde_json::Map::from(default.clone());
            let mut task_config = config.as_object().unwrap().clone();
            map_config.append(&mut task_config);
            let value = serde_json::Value::Object(map_config);
            log::info!("{:?}", &value);
            match serde_json::from_value(value) {
                Ok(config) => task_configs.push(config),
                Err(err) => {
                    log::error!("{:?}", &err);
                }
            }
        }
        log::info!("configs HttpRequestGenerator: {:?}", &configs);
        HttpRequestGenerator {
            root_config: configs,
            task_configs,
            handlebars: Handlebars::new(),
        }
    }
    pub fn get_url(
        &self,
        config: &HttpRequestJobConfig,
        context: &Value,
    ) -> Result<String, anyhow::Error> {
        // render without register
        self.handlebars
            .render_template(config.url_template.as_str(), context)
            .map_err(|err| anyhow!("{}", err))
        // register template using given name
        //reg.register_template_string("tpl_1", "Good afternoon, {{name}}")?;
        //println!("{}", reg.render("tpl_1", &json!({"name": "foo"}))?);
    }
}

impl HttpRequestGenerator {}
impl TaskApplicant for HttpRequestGenerator {
    fn get_name(&self) -> String {
        String::from("HttpRequest")
    }
    fn can_apply(&self, component: &ComponentInfo) -> bool {
        true
    }

    fn apply(
        &self,
        plan_id: &PlanId,
        component: &ComponentInfo,
        phase: JobRole,
    ) -> Result<Vec<Job>, Error> {
        let mut jobs = Vec::new();
        let mut context = json!({ "provider": component, "domain": DOMAIN.as_str() });
        if let Some(obj) = context["provider"].as_object_mut() {
            match component.component_type {
                ComponentType::Node => obj.insert(String::from("type"), Value::from("node")),
                ComponentType::Gateway => obj.insert(String::from("type"), Value::from("gw")),
            };
        }
        log::debug!(
            "Http Request apply for component {:?} with context {:?}",
            component,
            &context
        );
        for config in self.task_configs.iter() {
            if !config.can_apply(component, &phase) {
                debug!("Can not apply config {:?} for {:?}", config, component);
                continue;
            }
            match self.get_url(config, &context) {
                Ok(url) => {
                    let headers = config.generate_header(&self.handlebars, &context);
                    let body = config.generate_body(&self.handlebars, &context).ok();
                    let detail = JobHttpRequest {
                        url: url.clone(),
                        method: config.http_method.clone(),
                        headers,
                        body,
                        response_type: config.response.response_type.clone(),
                        response_values: config.response.values.clone(),
                    };
                    let mut job =
                        Job::new(plan_id.clone(), component, JobDetail::HttpRequest(detail));
                    job.parallelable = true;
                    job.component_url = url;
                    job.timeout = config.request_timeout;
                    job.repeat_number = config.repeat_number;
                    job.interval = config.interval;
                    jobs.push(job);
                }
                Err(err) => {
                    log::error!("{:?}", &err);
                }
            }
        }
        log::debug!("Generated {} jobs {:?}", jobs.len(), &jobs);
        Ok(jobs)
    }
    fn assign_jobs(
        &self,
        plan: &PlanModel,
        provider_node: &ComponentInfo,
        jobs: &Vec<Job>,
        workers: &MatchedWorkers,
    ) -> Result<Vec<JobAssignment>, anyhow::Error> {
        let mut assignments = Vec::default();
        jobs.iter().enumerate().for_each(|(ind, job)| {
            for worker in workers.best_workers.iter() {
                let job_assignment = JobAssignment::new(worker.clone(), job);
                assignments.push(job_assignment);
                debug!(
                    "Assign job {:?} to worker {:?}",
                    job.job_name,
                    worker.get_url("")
                )
            }
        });
        Ok(assignments)
    }
}
