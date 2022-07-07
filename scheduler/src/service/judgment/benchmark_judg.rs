use crate::models::job_result::ProviderTask;
use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::{JudgmentsResult, ReportCheck};
use crate::tasks::benchmark::generator::BenchmarkConfig;
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{BenchmarkResponse, JobBenchmarkResult, JobResultDetail, JobRole};
use common::jobs::{Job, JobResult};
use common::models::PlanEntity;
use common::tasks::LoadConfig;
use common::WorkerId;
use log::debug;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct BenchmarkResultCache {
    benchmarks: Mutex<HashMap<ProviderTask, ProviderBenchmarkResult>>,
}
#[derive(Clone, Debug, Default)]
pub struct ProviderBenchmarkResult {
    best_benchmark: BestBenchmarkResult,
    worker_benchmarks: HashMap<WorkerId, BenchmarkResponse>,
}
#[derive(Clone, Debug, Default)]
pub struct BestBenchmarkResult {
    pub result_count: usize,
    pub worker_id: WorkerId,
    pub response: BenchmarkResponse,
}

impl BestBenchmarkResult {
    pub fn new(result_count: usize, worker_id: WorkerId, response: BenchmarkResponse) -> Self {
        Self {
            result_count,
            worker_id,
            response,
        }
    }
    //Check if current is greater than other
    pub fn greater(&self, other: &BestBenchmarkResult, percentile: u32) -> bool {
        if let (Some(v1), Some(v2)) = (
            self.response.histograms.get(&percentile),
            other.response.histograms.get(&percentile),
        ) {
            return v1 > v2;
        }
        false
    }
}
impl ProviderBenchmarkResult {
    pub fn add_results(
        &mut self,
        benchmark_results: Vec<JobBenchmarkResult>,
        histogram_percentile: u32,
    ) -> Option<BestBenchmarkResult> {
        if benchmark_results.is_empty() {
            return None;
        }
        let first = benchmark_results.first().unwrap();
        let mut best_benchmark = BestBenchmarkResult::new(
            self.worker_benchmarks.len() + benchmark_results.len(),
            first.worker_id.clone(),
            first.response.clone(),
        );
        for result in benchmark_results.into_iter() {
            self.worker_benchmarks
                .insert(result.worker_id.clone(), result.response.clone());
            if let (Some(best), Some(cur)) = (
                best_benchmark
                    .response
                    .histograms
                    .get(&histogram_percentile),
                result.response.histograms.get(&histogram_percentile),
            ) {
                if best < cur {
                    best_benchmark.response = result.response.clone();
                    best_benchmark.worker_id = result.worker_id.clone();
                }
            }
        }
        if !self
            .best_benchmark
            .greater(&best_benchmark, histogram_percentile)
        {
            self.best_benchmark = best_benchmark;
        }
        Some(self.best_benchmark.clone())
    }
}
impl BenchmarkResultCache {
    pub async fn append_results(
        &self,
        provider_task: &ProviderTask,
        results: &Vec<JobResult>,
        histogram_percentile: u32,
    ) -> Option<BestBenchmarkResult> {
        if results.is_empty() {
            return None;
        }
        let mut benchmark_results = Vec::default();
        for res in results.iter() {
            if let JobResultDetail::Benchmark(benchmark_result) = &res.result_detail {
                benchmark_results.push(benchmark_result.clone());
            }
        }
        let mut povider_benchmark_result = self.benchmarks.lock().unwrap();

        povider_benchmark_result
            .entry(provider_task.clone())
            .or_insert(ProviderBenchmarkResult::default())
            .add_results(benchmark_results, histogram_percentile)
    }
}
#[derive(Debug)]
pub struct BenchmarkJudgment {
    result_service: Arc<JobResultService>,
    verification_config: BenchmarkConfig,
    regular_config: BenchmarkConfig,
    result_cache: BenchmarkResultCache,
}

impl BenchmarkJudgment {
    pub fn new(config_dir: &str, result_service: Arc<JobResultService>) -> Self {
        let verification_config = BenchmarkConfig::load_config(
            format!("{}/benchmark.json", config_dir).as_str(),
            &JobRole::Verification,
        );
        let regular_config = BenchmarkConfig::load_config(
            format!("{}/benchmark.json", config_dir).as_str(),
            &JobRole::Regular,
        );
        BenchmarkJudgment {
            result_service,
            verification_config,
            regular_config,
            result_cache: BenchmarkResultCache::default(),
        }
    }
}

#[async_trait]
impl ReportCheck for BenchmarkJudgment {
    fn get_name(&self) -> String {
        String::from("Benchmark")
    }

    fn can_apply_for_result(&self, task: &ProviderTask) -> bool {
        return task.task_name.as_str() == "Benchmark";
    }
    async fn apply(&self, _plan: &PlanEntity, _job: &Vec<Job>) -> Result<JudgmentsResult, Error> {
        //Todo: unimplement
        Ok(JudgmentsResult::Unfinished)
        /*
        let config = match JobRole::from_str(&*plan.phase)? {
            JobRole::Verification => &self.verification_config,
            JobRole::Regular => &self.regular_config,
        };

        let results = self.result_service.get_result_benchmarks(job).await?;

        info!("Benchmark results: {:?}", results);
        if results.is_empty() {
            return Ok(JudgmentsResult::Unfinished);
        }

        // Select result for judge
        let result = results
            .iter()
            .max_by(|r1, r2| r1.response_timestamp.cmp(&r2.response_timestamp))
            .unwrap();
        // Fixme: histogram config is dynamic.
        let result_response_time = result
            .response
            .histograms
            .get(&config.judge_histogram_percentile);

        info!("result_response_time: {:?}", result_response_time);

        return match result_response_time {
            None => Ok(JudgmentsResult::Error),
            Some(result_response_time) => {
                info!(
                    "Check result_response_time: {}, response_threshold: {}",
                    result_response_time, config.response_threshold
                );
                if (*result_response_time as Timestamp) > config.response_threshold {
                    Ok(JudgmentsResult::Failed)
                } else {
                    Ok(JudgmentsResult::Pass)
                }
            }
        };

             */
    }
    async fn apply_for_results(
        &self,
        provider_task: &ProviderTask,
        results: &Vec<JobResult>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        if results.is_empty() {
            return Ok(JudgmentsResult::Unfinished);
        }
        let phase = results.first().unwrap().phase.clone();
        let config = match phase {
            JobRole::Verification => &self.verification_config,
            JobRole::Regular => &self.regular_config,
        };
        if let Some(best_benchmark) = self
            .result_cache
            .append_results(provider_task, results, config.judge_histogram_percentile)
            .await
        {
            debug!(
                "Best benchmark result for provider {:?} is {:?}",
                provider_task.provider_id, &best_benchmark
            );
            if let Some(res) = best_benchmark
                .response
                .histograms
                .get(&config.judge_histogram_percentile)
            {
                debug!(
                    "Histogram value at {}% is {}. Config response time threshold {:?}",
                    &config.judge_histogram_percentile, res, config.response_threshold
                );
                if *res < config.response_threshold as f32 {
                    return Ok(JudgmentsResult::Pass);
                } else {
                    return Ok(JudgmentsResult::Failed);
                }
            }
        }

        Ok(JudgmentsResult::Unfinished)
    }
}
