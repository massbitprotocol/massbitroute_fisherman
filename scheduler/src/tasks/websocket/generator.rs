use crate::models::jobs::JobAssignmentBuffer;
use crate::persistence::PlanModel;
use crate::service::judgment::JudgmentsResult;
use crate::tasks::generator::TaskApplicant;
use crate::CONFIG;
use anyhow::{anyhow, Error};
use common::component::{ChainInfo, ComponentInfo, ComponentType};
use common::job_manage::{JobDetail, JobRole};
use common::jobs::{Job, JobAssignment};
use common::tasks::websocket_request::{JobWebsocket, JobWebsocketConfig};
use common::util::get_current_time;
use common::workers::MatchedWorkers;
use common::{PlanId, Timestamp, DOMAIN};
use handlebars::Handlebars;
use log::{debug, trace};
use serde_json::{json, Value};
use std::collections::HashMap;

/*
 * Periodically ping to node/gateway to get response time, to make sure node/gateway is working
 */
#[derive(Clone, Debug, Default)]
pub struct WebsocketGenerator {
    //root_config: serde_json::Map<String, serde_json::Value>,
    task_configs: Vec<JobWebsocketConfig>,
    handlebars: Handlebars<'static>,
}

impl WebsocketGenerator {
    pub fn get_name() -> String {
        String::from("Websocket")
    }
    pub fn new(config_dir: &str, phase: &JobRole) -> Self {
        let path = format!("{}/websocket.json", config_dir);
        let task_configs = JobWebsocketConfig::read_config(path.as_str(), phase);
        WebsocketGenerator {
            //root_config: configs,
            task_configs,
            handlebars: Handlebars::new(),
        }
    }
    pub fn get_url(
        &self,
        config: &JobWebsocketConfig,
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
    fn create_context(component: &ComponentInfo) -> Value {
        let mut context = json!({ "provider": component, "domain": DOMAIN.as_str() });
        if let Some(obj) = context["provider"].as_object_mut() {
            match component.component_type {
                ComponentType::Node => obj.insert(String::from("type"), Value::from("node")),
                ComponentType::Gateway => obj.insert(String::from("type"), Value::from("gw")),
            };
        };
        context
    }
    fn generate_job(
        &self,
        plan_id: &PlanId,
        component: &ComponentInfo,
        phase: JobRole,
        config: &JobWebsocketConfig,
        context: &Value,
    ) -> Result<Job, anyhow::Error> {
        self.get_url(config, context).map(|url| {
            let provider = &context["provider"];
            let chain_info = ChainInfo::new(
                provider["blockchain"]
                    .as_str()
                    .map(|str| str.to_string())
                    .unwrap_or_default(),
                provider["network"]
                    .as_str()
                    .map(|str| str.to_string())
                    .unwrap_or_default(),
            );
            let headers = config.generate_header(&self.handlebars, &context);
            let body = config.generate_body(&self.handlebars, &context).ok();
            let detail = JobWebsocket {
                url: url.clone(),
                chain_info: Some(chain_info.clone()),
                headers,
                body,
                response_type: config.response.response_type.clone(),
                response_values: config.response.values.clone(),
            };
            let mut job = Job::new(
                plan_id.clone(),
                Self::get_name(),
                config.name.clone(),
                component,
                JobDetail::Websocket(detail),
                phase,
            );
            job.parallelable = true;
            job.component_url = url;
            job.timeout = config.request_timeout;
            job.repeat_number = config.repeat_number;
            job.interval = config.interval;
            job
        })
    }
}
impl TaskApplicant for WebsocketGenerator {
    fn get_type(&self) -> String {
        String::from("Websocket")
    }
    fn can_apply(&self, _component: &ComponentInfo) -> bool {
        true
    }
    fn apply(
        &self,
        plan_id: &PlanId,
        component: &ComponentInfo,
        phase: JobRole,
        workers: &MatchedWorkers,
        _task_results: &HashMap<String, JudgmentsResult>,
    ) -> Result<JobAssignmentBuffer, Error> {
        let mut assignment_buffer = JobAssignmentBuffer::new();
        let context = Self::create_context(component);
        log::debug!(
            "Websocket apply for component {:?} with context {:?}",
            component,
            &context
        );
        for config in self.task_configs.iter().filter(|config| {
            config.match_phase(&phase)
                && config.match_blockchain(&component.blockchain)
                && config.match_network(&component.network)
                && config.match_provider_type(&component.component_type.to_string())
        }) {
            if !config.can_apply(component, &phase) {
                debug!("Can not apply config {:?} for {:?}", config, component);
                continue;
            }
            if let Ok(job) = self.generate_job(plan_id, component, phase.clone(), config, &context)
            {
                assignment_buffer.assign_job(job, workers, &Some(config.assignment.clone()));
            }
        }
        log::debug!(
            "Generated {:?} jobs and {:?} assignments.",
            &assignment_buffer.jobs.len(),
            &assignment_buffer.list_assignments.len()
        );
        Ok(assignment_buffer)
    }
    fn apply_with_cache(
        &self,
        plan_id: &PlanId,
        component: &ComponentInfo,
        phase: JobRole,
        workers: &MatchedWorkers,
        latest_update: HashMap<String, Timestamp>,
    ) -> Result<JobAssignmentBuffer, Error> {
        let mut assignment_buffer = JobAssignmentBuffer::new();
        let context = Self::create_context(component);
        log::debug!(
            "Websocket apply for component {:?} with context {:?}",
            component,
            &context
        );
        for config in self.task_configs.iter().filter(|config| {
            config.match_phase(&phase)
                && config.match_blockchain(&component.blockchain)
                && config.match_network(&component.network)
                && config.match_provider_type(&component.component_type.to_string())
        }) {
            if !config.can_apply(component, &phase) {
                trace!("Can not apply config {:?} for {:?}", config, component);
                continue;
            }
            let latest_update_timestamp = latest_update
                .get(&config.name)
                .map(|val| val.clone())
                .unwrap_or_default();
            //Check time_to_timeout > 0: timeout; <=0 not yet.
            let current_time = get_current_time();
            let timeout = current_time
                - (latest_update_timestamp
                    + config.interval
                    + CONFIG.generate_new_regular_timeout * 1000);
            if timeout > 0 {
                //Job for current config is already generated or received result recently
                debug!(
                    "time_to_timeout: {:?}. Generate task for {:?} with config {:?}",
                    timeout, component, config
                );
                if let Ok(job) =
                    self.generate_job(plan_id, component, phase.clone(), config, &context)
                {
                    assignment_buffer.assign_job(job, workers, &Some(config.assignment.clone()));
                }
            }
        }
        log::trace!("Generated jobs {:?}", &assignment_buffer);
        Ok(assignment_buffer)
    }
    fn assign_jobs(
        &self,
        _plan: &PlanModel,
        _provider_node: &ComponentInfo,
        jobs: &Vec<Job>,
        workers: &MatchedWorkers,
    ) -> Result<Vec<JobAssignment>, anyhow::Error> {
        let mut assignments = Vec::default();
        jobs.iter().enumerate().for_each(|(_ind, job)| {
            for worker in workers.measured_workers.iter() {
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
