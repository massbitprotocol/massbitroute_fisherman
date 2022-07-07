
use crate::models::job_result_cache::{JobResultCache, TaskKey, TaskResultCache};
use crate::report_processors::adapters::Appender;
use async_trait::async_trait;
use common::jobs::JobResult;
use common::util::get_current_time;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

const RESULT_CACHE_MAX_LENGTH: usize = 10;

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
    fn get_name(&self) -> String {
        "ResultCacheAppender".to_string()
    }
    async fn append_job_results(&self, results: &Vec<JobResult>) -> Result<(), anyhow::Error> {
        //log::info!("ResultCacheAppender append results");
        if results.is_empty() {
            return Ok(());
        }

        {
            let mut result_cache = self.result_cache.lock().await;
            for result in results {
                let component_id = &result.provider_id;

                let task_key = TaskKey {
                    task_type: result.result_detail.get_name(),
                    task_name: result.job_name.clone(),
                };
                // Create new entry if need
                let result_by_task = result_cache
                    .result_cache_map
                    .entry(component_id.clone())
                    .or_insert(HashMap::new());
                let task_result_cache = result_by_task
                    .entry(task_key)
                    .or_insert(TaskResultCache::new(get_current_time()));
                // Store to cache
                task_result_cache.push_back_cache(result.clone());

                while task_result_cache.len() > RESULT_CACHE_MAX_LENGTH {
                    task_result_cache.pop_front();
                }
            }
            log::debug!(
                "{} result_cache: {:?}",
                result_cache.result_cache_map.len(),
                result_cache.result_cache_map
            );
        }
        Ok(())
    }
    /*
    async fn append_ping_results(&self, results: &Vec<JobPingResult>) -> Result<(), Error> {
        log::debug!("ResultCacheAppender append ping results");
        if results.is_empty() {
            return Ok(());
        }
        {
            let mut result_cache = self.result_cache.lock().await;
            for result in results {
                let component_id = &result.job.component_id;
                let job_result =
                    JobResult::new(JobResultDetail::Ping(result.clone()), None, &Job::default());
                let result_by_task = result_cache
                    .result_cache_map
                    .entry(component_id.clone())
                    .or_insert(HashMap::new());
                let task_key = TaskKey {
                    task_type: result.job.job_name.clone(),
                    task_name: result.job.job_name.clone(),
                };
                let e = result_by_task
                    .entry(task_key)
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
                let job_result = JobResult::new(
                    JobResultDetail::LatestBlock(result.clone()),
                    None,
                    &Job::default(),
                );
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
                let job_result = JobResult::new(
                    JobResultDetail::Benchmark(result.clone()),
                    None,
                    &Job::default(),
                );
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
     */
}
