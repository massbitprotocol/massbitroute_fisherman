use crate::job_manage::{Job, JobDetail, JobResult};
use crate::logger::helper::message;
use crate::tasks::eth::{CallLatestBlockError, JobLatestBlockResult, LatestBlockResponse};
use crate::tasks::executor::TaskExecutor;
use crate::util::get_current_time;
use crate::{task_spawn, Timestamp, WorkerId};
use anyhow::Error;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::header::HeaderMap;
use reqwest::{get, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::format;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

#[derive(Clone, Debug, Default)]
pub struct LatestBlockExecutor {
    worker_id: WorkerId,
    client: Client,
}

impl LatestBlockExecutor {
    pub fn new(worker_id: WorkerId) -> Self {
        LatestBlockExecutor {
            worker_id,
            client: reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap(),
        }
    }

    pub async fn call_latest_block(
        &self,
        job: &Job,
    ) -> Result<LatestBlockResponse, CallLatestBlockError> {
        //Get job detail
        let job_detail = match job
            .job_detail
            .as_ref()
            .ok_or(CallLatestBlockError::GetBodyError(
                "No job detail".to_string(),
            ))? {
            JobDetail::LatestBlock(job_detail) => Ok(job_detail),
            _ => Err(CallLatestBlockError::GetBodyError(
                "Wrong job detail type".to_string(),
            )),
        }?;

        match &job.job_detail {
            Some(JobDetail::LatestBlock(latest_block)) => {}
            None => {}
            _ => {}
        }
        // Measure response_time
        let mut builder = self
            .client
            .post(job.component_url.as_str())
            .timeout(Duration::from_millis(job.timeout as u64));
        // Add header
        for (key, value) in job.header.iter() {
            builder = builder.header(key, value);
        }

        let body = job_detail.request_body.clone();

        let now = Instant::now();
        debug!("call_latest_block builder: {:?}", builder);
        let resp = builder
            .body(body)
            .send()
            .await
            .map_err(|err| CallLatestBlockError::SendError(format!("{}", err)))?;

        let http_code = resp.status().as_u16();
        debug!("call_latest_block http_code: {}", http_code);
        let response_body = resp
            .text()
            .await
            .map_err(|err| CallLatestBlockError::GetBodyError(format!("{}", err)))?;
        //debug!("call_latest_block response_body: {}", response_body);
        let response_time = now.elapsed();

        let BlockData {
            block_number,
            block_timestamp,
            block_hash,
        } = Self::parse_block_data(&response_body)
            .map_err(|err| CallLatestBlockError::GetBodyError(format!("{}", err)))?;

        let latest_block_result = LatestBlockResponse {
            response_time: response_time.as_millis() as i64,
            block_number,
            block_timestamp,
            block_hash,
            http_code,
            error_code: 0,
            message: "success".to_string(),
            chain_info: job_detail.chain_info.clone(),
        };
        Ok(latest_block_result)
    }

    fn parse_block_data(body: &String) -> Result<BlockData, Error> {
        let body: Value = serde_json::from_str(body)
            .map_err(|e| Error::msg(format!("Err {} when parsing response", e)))?;

        // get result
        let result = body
            .get("result")
            .ok_or(Error::msg("Cannot get `result` key"))?;
        let block_hash = result
            .get("hash")
            .ok_or(Error::msg("Cannot get `hash` key"))?
            .to_string();
        let block_number = result
            .get("number")
            .ok_or(Error::msg("Cannot get `number` key"))?
            .to_string();
        let block_timestamp = result
            .get("timestamp")
            .ok_or(Error::msg("Cannot get `timestamp` key"))?
            .to_string();

        let block_hash = block_hash
            .trim_start_matches("\"0x")
            .trim_end_matches("\"")
            .to_string();
        let block_number = block_number
            .trim_start_matches("\"0x")
            .trim_end_matches("\"");
        let block_timestamp = block_timestamp
            .trim_start_matches("\"0x")
            .trim_end_matches("\"");
        info!(
            "result: {}, block_hash: {}, block_number: {}, block_timestamp: {}",
            result, block_hash, block_number, block_timestamp
        );

        Ok(BlockData {
            block_number: u64::from_str_radix(block_number, 16)?,
            block_timestamp: i64::from_str_radix(block_timestamp, 16)?,
            block_hash,
        })
    }
}

#[async_trait]
impl TaskExecutor for LatestBlockExecutor {
    async fn execute(&self, job: &Job, result_sender: Sender<JobResult>) -> Result<(), Error> {
        let job_detail = match job
            .job_detail
            .as_ref()
            .ok_or(Error::msg("No job detail".to_string()))?
        {
            JobDetail::LatestBlock(job_detail) => Ok(job_detail),
            _ => Err(Error::msg("Wrong job detail type".to_string())),
        }?;

        info!("TaskLatestBlock execute for job {:?}", &job);
        let res = self.call_latest_block(job).await;
        let response = match res {
            Ok(res) => res,
            Err(err) => LatestBlockResponse::new_error(
                err.get_code(),
                err.get_message().as_str(),
                job_detail.chain_info.clone(),
            ),
        };
        info!("LatestBlock result {:?}", &response);
        let latest_block_result = JobLatestBlockResult {
            job: job.clone(),
            worker_id: self.worker_id.clone(),
            response,
            execution_timestamp: get_current_time(),
        };
        let res = result_sender
            .send(JobResult::LatestBlock(latest_block_result))
            .await;
        info!("send res: {:?}", res);

        Ok(())
    }
    fn can_apply(&self, job: &Job) -> bool {
        debug!("can_apply LatestBlockExecutor job: {:?}", job.job_detail);
        return match job.job_detail.as_ref() {
            None => false,
            Some(job_detail) => match job_detail {
                JobDetail::LatestBlock(_) => true,
                _ => false,
            },
        };
    }
}

struct BlockData {
    pub block_number: u64,
    pub block_timestamp: Timestamp,
    pub block_hash: String,
}
