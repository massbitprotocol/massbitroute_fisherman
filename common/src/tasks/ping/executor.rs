use crate::job_manage::{JobDetail, JobResultDetail};
use crate::jobs::Job;
use crate::logger::helper::message;
use crate::tasks::executor::TaskExecutor;
use crate::tasks::ping::{CallPingError, JobPingResult, PingResponse};
use crate::util::get_current_time;
use crate::{task_spawn, WorkerId};
use anyhow::Error;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::{get, Client};
use serde::{Deserialize, Serialize};
use std::fmt::format;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

#[derive(Clone, Debug, Default)]
pub struct PingExecutor {
    worker_id: WorkerId,
    client: Client,
}

impl PingExecutor {
    pub fn new(worker_id: WorkerId) -> Self {
        PingExecutor {
            worker_id,
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
            .timeout(Duration::from_millis(job.timeout as u64))
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
            response_time: response_time.as_millis() as i64,
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
    async fn execute(
        &self,
        job: &Job,
        result_sender: Sender<JobResultDetail>,
    ) -> Result<(), Error> {
        debug!("TaskPing execute for job {:?}", &job);
        let res = self.call_ping(job).await;
        let response = match res {
            Ok(res) => res,
            Err(err) => err.into(),
        };
        debug!("Ping result {:?}", &response);
        let ping_result = JobPingResult {
            job: job.clone(),
            worker_id: self.worker_id.clone(),
            response,
        };
        let res = result_sender.send(JobResultDetail::Ping(ping_result)).await;
        debug!("send res: {:?}", res);

        Ok(())
    }
    fn can_apply(&self, job: &Job) -> bool {
        return match job.job_detail.as_ref() {
            Some(JobDetail::Ping(_)) => true,
            _ => false,
        };
    }
}
