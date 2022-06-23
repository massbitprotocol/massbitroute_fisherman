use crate::models::job_result_cache::{JobResultCache, TaskResultCache};
use crate::report_processors::adapters::Appender;
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobBenchmarkResult, JobResultDetail};
use common::jobs::JobResult;
use common::tasks::eth::JobLatestBlockResult;
use common::tasks::ping::JobPingResult;
use common::util::get_current_time;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;

const RESULT_CACHE_MAX_LENGTH: usize = 3;

pub struct ResultCacheAppender {
    result_cache: Arc<Mutex<JobResultCache>>,
}

impl ResultCacheAppender {
    pub fn new(result_cache: Arc<Mutex<JobResultCache>>) -> Self {
        ResultCacheAppender { result_cache }
    }
}

#[async_trait]
impl Appender for ResultCacheAppender {
    async fn append_ping_results(&self, results: &Vec<JobPingResult>) -> Result<(), Error> {
        log::debug!("ResultCacheAppender append ping results");
        if results.is_empty() {
            return Ok(());
        }
        {
            let mut result_cache = self.result_cache.lock().await;
            for result in results {
                let component_id = &result.job.component_id;
                let job_result = JobResult::new(JobResultDetail::Ping(result.clone()), None);
                let result_by_task = result_cache
                    .result_cache_map
                    .entry(component_id.clone())
                    .or_insert(HashMap::new());
                let e = result_by_task
                    .entry(result.job.job_name.clone())
                    .or_insert(TaskResultCache::new(get_current_time()));
                e.push_back(job_result);
                while e.len() > RESULT_CACHE_MAX_LENGTH {
                    e.pop_front();
                }
            }
        }
        Ok(())
    }
    async fn append_latest_block_results(
        &self,
        results: &Vec<JobLatestBlockResult>,
    ) -> Result<(), anyhow::Error> {
        log::debug!("ResultCacheAppender append latest block results");
        if results.is_empty() {
            return Ok(());
        }
        {
            let mut result_cache = self.result_cache.lock().await;
            for result in results {
                let component_id = &result.job.component_id;
                let job_result = JobResult::new(JobResultDetail::LatestBlock(result.clone()), None);
                let result_by_task = result_cache
                    .result_cache_map
                    .entry(component_id.clone())
                    .or_insert(HashMap::new());
                let e = result_by_task
                    .entry(result.job.job_name.clone())
                    .or_insert(TaskResultCache::new(get_current_time()));
                e.push_back(job_result);
                while e.len() > RESULT_CACHE_MAX_LENGTH {
                    e.pop_front();
                }
            }
        }
        Ok(())
    }
    async fn append_benchmark_results(
        &self,
        results: &Vec<JobBenchmarkResult>,
    ) -> Result<(), anyhow::Error> {
        log::debug!("ResultCacheAppender append benchmark results");
        if results.is_empty() {
            return Ok(());
        }
        {
            let mut result_cache = self.result_cache.lock().await;
            for result in results {
                let component_id = &result.job.component_id;
                let job_result = JobResult::new(JobResultDetail::Benchmark(result.clone()), None);
                let result_by_task = result_cache
                    .result_cache_map
                    .entry(component_id.clone())
                    .or_insert(HashMap::new());
                let e = result_by_task
                    .entry(result.job.job_name.clone())
                    .or_insert(TaskResultCache::new(get_current_time()));
                e.push_back(job_result);
                while e.len() > RESULT_CACHE_MAX_LENGTH {
                    e.pop_front();
                }
            }
        }
        Ok(())
    }
}
