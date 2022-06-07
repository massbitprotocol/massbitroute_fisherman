use crate::job_manage::{BenchmarkResponse, Job, JobBenchmarkResult, JobResult};
use crate::logger::helper::message;
use crate::task_spawn;
use crate::tasks::eth::CallBenchmarkError;
use crate::tasks::executor::TaskExecutor;
use crate::tasks::get_current_time;
use anyhow::Error;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt::format;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

#[derive(Clone, Debug, Default)]
pub struct BenchmarkExecutor {
    client: Client,
}

impl BenchmarkResponse {
    pub fn new_error(error_code: u32, message: &str) -> Self {
        BenchmarkResponse {
            request_rate: 0.0,
            transfer_rate: 0.0,
            average_latency: 0.0,
            histograms: Default::default(),
            error_code,
            message: message.to_string(),
        }
    }
}

impl BenchmarkExecutor {
    pub fn new() -> Self {
        BenchmarkExecutor {
            client: Default::default(),
        }
    }
    pub async fn call_benchmark(&self, job: &Job) -> Result<BenchmarkResponse, CallBenchmarkError> {
        Ok(Default::default())
    }
}

#[async_trait]
impl TaskExecutor for BenchmarkExecutor {
    async fn execute(
        &self,
        job: &Job,
        result_sender: Sender<JobResult>,
        newjob_sender: Sender<Job>,
    ) -> Result<(), Error> {
        debug!("TaskBenchmark execute for job {:?}", &job);
        let executor = self.clone();
        let job = job.clone();

        Ok(())
    }
}
