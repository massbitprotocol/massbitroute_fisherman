use crate::job_manage::{Job, JobPingResult, JobResult, PingResponse};
use crate::logger::helper::message;
use crate::tasks::executor::TaskExecutor;
use crate::tasks::get_current_time;
use crate::tasks::ping::CallPingError;
use crate::{task_spawn, Timestamp};
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
pub struct PingExecutor {
    client: Client,
}

impl PingResponse {
    pub fn new_error(error_code: u32, message: &str) -> Self {
        PingResponse {
            response_time: 0,
            response_body: "".to_string(),
            http_code: 0,
            error_code,
            message: message.to_string(),
        }
    }
}

impl PingExecutor {
    pub fn new() -> Self {
        PingExecutor {
            client: reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap(),
        }
    }
    pub async fn call_ping(&self, job: &Job) -> Result<PingResponse, CallPingError> {
        // Measure response_time
        let now = Instant::now();
        let resp = self
            .client
            .get(job.component_url.as_str())
            .timeout(Duration::from_millis(job.time_out as u64))
            .send()
            .await
            .map_err(|err| CallPingError::SendError(format!("{}", err)))?;
        let http_code = resp.status().as_u16();
        let response_body = resp
            .text()
            .await
            .map_err(|err| CallPingError::GetBodyError(format!("{}", err)))?;

        let response_time = now.elapsed();

        let ping_result = PingResponse {
            response_time: response_time.as_millis(),
            response_body,
            http_code,
            error_code: 0,
            message: "success".to_string(),
        };
        Ok(ping_result)
    }
}

#[async_trait]
impl TaskExecutor for PingExecutor {
    async fn execute(&self, job: &Job, sender: Sender<JobResult>) -> Result<(), Error> {
        debug!("TaskPing execute for job {:?}", &job);
        let executor = self.clone();
        let job = job.clone();
        let mut responses = Vec::new();
        for count in 0..job.repeat_number {
            info!("** Do ping {} **", count);
            let res = executor.call_ping(&job).await;
            info!("Ping result {:?}", res);
            let res = match res {
                Ok(res) => res,
                Err(err) => err.into(),
            };
            responses.push(res);
            sleep(Duration::from_millis(job.interval as u64));
        }

        let ping_result = JobPingResult {
            job,
            response_timestamp: get_current_time(),
            responses,
        };
        let res = sender.send(JobResult::Ping(ping_result)).await;
        debug!("send res: {:?}", res);
        Ok(())
    }
}
