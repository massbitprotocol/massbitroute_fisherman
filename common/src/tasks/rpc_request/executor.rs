use crate::job_manage::JobResultDetail;
use crate::jobs::{Job, JobResult};
use crate::logger::helper::message;
use crate::task_spawn;
use crate::tasks::executor::TaskExecutor;
use crate::tasks::ping::{CallPingError, JobPingResult};
use crate::tasks::rpc_request::{JobRpcResponse, JobRpcResult, RpcRequestError};
use crate::util::get_current_time;
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
pub struct RpcRequestExecutor {
    client: Client,
}

impl RpcRequestExecutor {
    pub fn new() -> Self {
        RpcRequestExecutor {
            client: reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap(),
        }
    }
    pub async fn call_ping(&self, job: &Job) -> Result<JobRpcResponse, RpcRequestError> {
        // Measure response_time
        let now = Instant::now();
        let resp = self
            .client
            .get(job.component_url.as_str())
            .timeout(Duration::from_millis(job.timeout as u64))
            .send()
            .await
            .map_err(|err| RpcRequestError::SendError(format!("{}", err)))?;
        let http_code = resp.status().as_u16();
        let response_body = resp
            .text()
            .await
            .map_err(|err| RpcRequestError::GetBodyError(format!("{}", err)))?;

        let response_time = now.elapsed();

        let ping_result = JobRpcResponse {
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
impl TaskExecutor for RpcRequestExecutor {
    async fn execute(&self, job: &Job, result_sender: Sender<JobResult>) -> Result<(), Error> {
        debug!("TaskPing execute for job {:?}", &job);
        let res = self.call_ping(job).await;
        let response = match res {
            Ok(res) => res,
            Err(err) => err.into(),
        };
        debug!("Rpc result {:?}", &response);
        let result = JobRpcResult {
            job: job.clone(),
            //response_timestamp: get_current_time(),
            response,
        };
        let job_result = JobResult::new(JobResultDetail::RpcRequest(result), None, job);
        let res = result_sender.send(job_result).await;
        debug!("send res: {:?}", res);
        Ok(())
    }

    fn can_apply(&self, job: &Job) -> bool {
        true
    }
}
