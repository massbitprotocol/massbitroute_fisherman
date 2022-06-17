/*
 * Check from any gateway can connection to any node
 */

use crate::models::tasks::generator::TaskApplicant;
use common::component::ComponentInfo;
use common::job_manage::{JobBenchmark, JobDetail, JobRole};
use common::jobs::Job;
use common::models::PlanEntity;
use common::tasks::LoadConfig;
use common::{PlanId, Timestamp};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct BenchmarkGenerator {
    config: BenchmarkConfig,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct BenchmarkConfig {
    benchmark_thread: u32,
    benchmark_connection: u32,
    benchmark_duration: Timestamp,
    benchmark_rate: u32,
    script: String,
    histograms: Vec<u32>,
    url_path: String,
    pub judge_histogram_percentile: u32,
    pub response_threshold: Timestamp,
}

impl LoadConfig<BenchmarkConfig> for BenchmarkConfig {}

impl BenchmarkGenerator {
    pub fn new(config_dir: &str, role: &JobRole) -> Self {
        BenchmarkGenerator {
            config: BenchmarkConfig::load_config(
                format!("{}/benchmark.json", config_dir).as_str(),
                role,
            ),
        }
    }
    pub fn get_url(&self, component: &ComponentInfo) -> String {
        format!("https://{}/_ping", component.ip)
    }
}
impl TaskApplicant for BenchmarkGenerator {
    fn can_apply(&self, component: &ComponentInfo) -> bool {
        true
    }

    fn apply(
        &self,
        plan_id: &PlanId,
        component: &ComponentInfo,
    ) -> Result<Vec<Job>, anyhow::Error> {
        log::debug!("TaskPing apply for component {:?}", component);
        let job_benchmark = JobBenchmark {
            component_type: component.component_type.clone(),
            chain_type: component.blockchain.clone(),
            connection: self.config.benchmark_connection,
            thread: self.config.benchmark_thread,
            rate: self.config.benchmark_rate,
            duration: self.config.benchmark_duration,
            script: self.config.script.clone(),
            histograms: self.config.histograms.clone(),
            url_path: self.config.url_path.clone(),
        };
        let job_detail = JobDetail::Benchmark(job_benchmark);
        let mut job = Job::new(plan_id.clone(), component, job_detail);
        job.component_url = self.get_url(component);
        let vec = vec![job];
        Ok(vec)
    }
}