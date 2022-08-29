/*
 * Check from any gateway can connection to any node
 */

use crate::models::job_result_cache::TaskKey;
use crate::models::jobs::JobAssignmentBuffer;

use crate::service::judgment::JudgmentsResult;
use crate::tasks::generator::TaskApplicant;

use common::component::{ComponentInfo, ComponentType};
use common::job_manage::{JobBenchmark, JobDetail, JobRole};
use common::jobs::{AssignmentConfig, Job};
use common::tasks::{LoadConfigs, TaskConfigTrait};
use common::workers::MatchedWorkers;
use common::{NetworkType, PlanId, Timestamp, DOMAIN};

use crate::{TemplateRender, CONFIG_BENCHMARK_DIR};
use handlebars::Handlebars;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct BenchmarkGenerator {
    configs: Vec<BenchmarkConfig>,
    handlebars: Handlebars<'static>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct BenchmarkConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub phases: Vec<String>,
    #[serde(default)]
    pub provider_types: Vec<String>,
    #[serde(default)]
    pub blockchains: Vec<String>,
    #[serde(default)]
    pub networks: Vec<String>,
    #[serde(default)]
    benchmark_thread: u32,
    #[serde(default)]
    benchmark_connection: u32,
    #[serde(default)]
    benchmark_duration: Timestamp,
    #[serde(default)]
    benchmark_rate: u32,
    #[serde(default)]
    timeout: Timestamp,
    #[serde(default)]
    script: String,
    #[serde(default)]
    histograms: Vec<u32>,
    #[serde(default)]
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

impl TaskConfigTrait for BenchmarkConfig {
    fn match_phase(&self, phase: &JobRole) -> bool {
        self.phases.contains(&String::from("*")) || self.phases.contains(&phase.to_string())
    }

    fn get_blockchain(&self) -> &Vec<String> {
        &self.blockchains
    }

    fn match_network(&self, network: &NetworkType) -> bool {
        let network = network.to_lowercase();
        if !self.networks.contains(&String::from("*")) && !self.networks.contains(&network) {
            log::trace!(
                "Networks {:?} not match with {:?}",
                &network,
                &self.networks
            );
            return false;
        }
        true
    }
    fn match_provider_type(&self, provider_type: &String) -> bool {
        let provider_type = provider_type.to_lowercase();
        if !self.provider_types.contains(&String::from("*"))
            && !self.provider_types.contains(&provider_type)
        {
            log::trace!(
                "Provider type {:?} not match with {:?}",
                &provider_type,
                &self.networks
            );
            return false;
        }
        true
    }
    fn can_apply(&self, provider: &ComponentInfo, phase: &JobRole) -> bool {
        if !self.active {
            return false;
        }
        // Check phase
        if !self.match_phase(phase) {
            return false;
        }
        if !self.match_provider_type(&provider.component_type.to_string()) {
            return false;
        }
        if !self.match_blockchain(&provider.blockchain) {
            return false;
        }
        if !self.match_network(&provider.network) {
            return false;
        }
        true
    }
}

impl LoadConfigs<BenchmarkConfig> for BenchmarkConfig {}

impl BenchmarkGenerator {
    pub fn get_name() -> String {
        String::from("Benchmark")
    }
    pub fn new(config_dir: &str, role: &JobRole) -> Self {
        // let path = format!("{}/benchmark", config_dir);
        let path = Path::new(config_dir).join(&*CONFIG_BENCHMARK_DIR);
        let configs: Vec<BenchmarkConfig> = BenchmarkConfig::read_configs(&path, role);
        log::debug!("Benchmark config {:?}", &configs);
        BenchmarkGenerator {
            configs,
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
    pub fn generate_job(
        &self,
        plan_id: &PlanId,
        component: &ComponentInfo,
        phase: JobRole,
        config: &BenchmarkConfig,
        context: &Value,
    ) -> Result<Job, anyhow::Error> {
        BenchmarkConfig::generate_url(config.url_template.as_str(), &self.handlebars, context).map(
            |job_url| {
                let headers =
                    BenchmarkConfig::generate_header(&config.headers, &self.handlebars, &context);
                let body =
                    BenchmarkConfig::generate_body(&config.body, &self.handlebars, &context).ok();
                let job_benchmark = JobBenchmark {
                    component_type: component.component_type.clone(),
                    chain_type: component.blockchain.clone(),
                    connection: config.benchmark_connection,
                    thread: config.benchmark_thread,
                    rate: config.benchmark_rate,
                    timeout: config.timeout,
                    duration: config.benchmark_duration,
                    script: config.script.clone(),
                    histograms: config.histograms.clone(),
                    url_path: job_url.clone(),
                    method: config.http_method.clone(),
                    headers,
                    body,
                };
                let mut job = Job::new(
                    plan_id.clone(),
                    Self::get_name(),
                    config.name.clone(),
                    component,
                    JobDetail::Benchmark(job_benchmark),
                    phase,
                );
                job.parallelable = false;
                job.component_url = job_url;
                job.timeout = config.timeout;
                job.repeat_number = 0;
                job.interval = 0;
                job
            },
        )
    }
}
impl TaskApplicant for BenchmarkGenerator {
    fn get_type(&self) -> String {
        String::from("Benchmark")
    }
    fn get_task_names(&self) -> Vec<String> {
        self.configs
            .iter()
            .map(|config| config.name.clone())
            .collect()
    }
    fn has_all_dependent_results(
        &self,
        _plan_id: &PlanId,
        results: &HashMap<TaskKey, JudgmentsResult>,
    ) -> bool {
        for config in self.configs.iter() {
            if let Some(dependencies) = config.dependencies.as_ref() {
                for (key, values) in dependencies {
                    for value in values {
                        let result_key = TaskKey {
                            task_type: key.clone(),
                            task_name: value.clone(),
                        };
                        let has_result = results.contains_key(&result_key);
                        if !has_result {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
    //Fixme: Improve task dependency
    // fn get_task_dependencies(&self) -> TaskDependency {
    //     let mut task_dependencies = TaskDependency::default();
    //     for config in self.configs.iter() {
    //         if let Some(dependencies) = config.dependencies.as_ref() {
    //             for (key, values) in dependencies {
    //                 let hash_set = task_dependencies
    //                     .entry(key.clone())
    //                     .or_insert(HashSet::default());
    //                 for value in values {
    //                     hash_set.insert(value.clone());
    //                 }
    //             }
    //         }
    //     }
    //     task_dependencies
    // }
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
    ) -> Result<JobAssignmentBuffer, anyhow::Error> {
        let mut assignment_buffer = JobAssignmentBuffer::default();
        log::debug!("Task benchmark apply for component {:?}", component);
        log::debug!("Workers {:?}", workers);
        let context = Self::create_context(component);
        log::debug!(
            "Benchmark apply for component {:?} with context {:?}",
            component,
            &context
        );
        for config in self.configs.iter().filter(|config| {
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
                assignment_buffer.assign_job(job, workers, &config.assignment);
            }
        }
        debug!(
            "Generated {} benchmark jobs and {} assignments: {:?}.",
            &assignment_buffer.jobs.len(),
            &assignment_buffer.list_assignments.len(),
            &assignment_buffer.list_assignments
        );

        Ok(assignment_buffer)
    }
}
