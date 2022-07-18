/*
 * Check from any gateway can connection to any node
 */

use crate::models::jobs::JobAssignmentBuffer;
use crate::models::TaskDependency;
use crate::tasks::generator::TaskApplicant;
use anyhow::anyhow;
use common::component::{ComponentInfo, ComponentType};
use common::job_manage::{JobBenchmark, JobDetail, JobRole};
use common::jobs::{AssignmentConfig, Job};
use common::tasks::{LoadConfig, TemplateRender};
use common::workers::MatchedWorkers;
use common::{PlanId, Timestamp, DOMAIN};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct BenchmarkGenerator {
    configs: Vec<BenchmarkConfig>,
    handlebars: Handlebars<'static>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct BenchmarkConfig {
    benchmark_thread: u32,
    benchmark_connection: u32,
    benchmark_duration: Timestamp,
    benchmark_rate: u32,
    #[serde(default)]
    timeout: Option<u32>,
    script: String,
    histograms: Vec<u32>,
    url_template: String,
    #[serde(default)]
    pub http_method: String,
    pub headers: serde_json::Map<String, serde_json::Value>,
    pub body: serde_json::Value,
    pub judge_histogram_percentile: u32,
    pub response_threshold: Timestamp,
    pub assignment: Option<AssignmentConfig>,
    pub dependencies: Option<HashMap<String, Vec<String>>>,
}

impl TemplateRender for BenchmarkConfig {}
impl LoadConfig<BenchmarkConfig> for BenchmarkConfig {}

impl BenchmarkGenerator {
    pub fn get_name() -> String {
        String::from("Benchmark")
    }
    pub fn new(config_dir: &str, role: &JobRole) -> Self {
        let config: BenchmarkConfig =
            BenchmarkConfig::load_config(format!("{}/benchmark.json", config_dir).as_str(), role);
        log::debug!("Benchmark config {:?}", &config);
        BenchmarkGenerator {
            config,
            handlebars: Handlebars::new(),
        }
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
    pub fn get_url(&self, component: &ComponentInfo) -> Result<String, anyhow::Error> {
        let context = Self::create_context(component);
        self.handlebars
            .render_template(self.config.url_template.as_str(), &context)
            .map_err(|err| anyhow!("{}", err))
    }
}
impl TaskApplicant for BenchmarkGenerator {
    fn get_name(&self) -> String {
        String::from("Benchmark")
    }
    fn get_task_dependencies(&self) -> TaskDependency {
        self.config
            .dependencies
            .as_ref()
            .map(|dep| dep.clone())
            .unwrap_or_default()
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
    ) -> Result<JobAssignmentBuffer, anyhow::Error> {
        log::debug!("Task benchmark apply for component {:?}", component);
        log::debug!("Workers {:?}", workers);
        let context = Self::create_context(component);
        let job_url = self.get_url(component)?;
        let headers =
            BenchmarkConfig::generate_header(&self.config.headers, &self.handlebars, &context);
        let body =
            BenchmarkConfig::generate_body(&self.config.body, &self.handlebars, &context).ok();
        let job_benchmark = JobBenchmark {
            component_type: component.component_type.clone(),
            chain_type: component.blockchain.clone(),
            connection: self.config.benchmark_connection,
            thread: self.config.benchmark_thread,
            rate: self.config.benchmark_rate,
            timeout: self.config.timeout,
            duration: self.config.benchmark_duration,
            script: self.config.script.clone(),
            histograms: self.config.histograms.clone(),
            url_path: job_url.clone(),
            method: self.config.http_method.clone(),
            headers,
            body,
        };

        let job_detail = JobDetail::Benchmark(job_benchmark);
        let mut job = Job::new(
            plan_id.clone(),
            Self::get_name(),
            job_detail.get_job_name(),
            component,
            job_detail,
            phase,
        );
        job.component_url = job_url;
        job.header.insert(
            "host".to_lowercase().to_string(),
            component.get_host_header(&*DOMAIN),
        );
        job.header.insert(
            "x-api-key".to_lowercase().to_string(),
            component.token.clone(),
        );
        let mut assignment_buffer = JobAssignmentBuffer::default();
        assignment_buffer.assign_job(job, workers, &self.config.assignment);
        Ok(assignment_buffer)
    }
}
