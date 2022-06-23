use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobDetail, JobResultDetail};
use common::jobs::{Job, JobResult};
use common::logger::helper::message;
use common::tasks::executor::TaskExecutor;
use common::tasks::http_request::{
    HttpRequestError, JobHttpResponse, JobHttpResponseDetail, JobHttpResult,
};
use common::tasks::ping::{CallPingError, JobPingResult};
use common::tasks::rpc_request::{JobRpcResponse, JobRpcResult, RpcRequestError};
use common::util::get_current_time;
use common::{task_spawn, WorkerId};
use log::{debug, error, info};
use reqwest::{get, Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fmt::format;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

#[derive(Clone, Debug, Default)]
pub struct HttpRequestExecutor {
    worker_id: WorkerId,
    client: Client,
}

impl HttpRequestExecutor {
    pub fn new(worker_id: WorkerId) -> Self {
        HttpRequestExecutor {
            worker_id,
            client: reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap(),
        }
    }
    pub async fn call_http_request(&self, job: &Job) -> Result<JobHttpResponse, HttpRequestError> {
        // Measure response_time
        let now = Instant::now();
        if let Some(JobDetail::HttpRequest(request)) = job.job_detail.as_ref() {
            let mut req_builder = match request.method.to_lowercase().as_str() {
                "post" => self.client.post(job.component_url.as_str()),
                "put" => self.client.put(job.component_url.as_str()),
                "patch" => self.client.patch(job.component_url.as_str()),
                "get" => self.client.get(job.component_url.as_str()),
                _ => return Err(HttpRequestError::BuildError("Method invalid".to_string())),
            };
            req_builder = req_builder.timeout(Duration::from_millis(job.timeout as u64));
            // Add header
            for (key, value) in request.headers.iter() {
                req_builder = req_builder.header(key, value);
            }
            //Body
            if let Some(body) = &request.body {
                let req_body = body.clone().to_string();
                log::debug!("Request body {:?}", &req_body);
                req_builder = req_builder.body(req_body);
            }
            let resp = req_builder
                .send()
                .await
                .map_err(|err| HttpRequestError::SendError(format!("{}", err)))?;
            let http_code = resp.status().as_u16();

            // let response_body = resp
            //     .json()
            //     .await
            //     .map_err(|err| HttpRequestError::GetBodyError(format!("{}", err)))?;

            let response_time = now.elapsed();
            let response_detail = self
                .parse_response(resp, &request.response_type, &request.response_values)
                .await;
            match response_detail {
                Ok(detail) => Ok(JobHttpResponse {
                    response_time: response_time.as_millis() as i64,
                    detail,
                    http_code,
                    error_code: 0,
                    message: "success".to_string(),
                }),
                Err(err) => {
                    error!("{:?}", &err);
                    Ok(JobHttpResponse {
                        response_time: response_time.as_millis() as i64,
                        detail: JobHttpResponseDetail::default(),
                        http_code,
                        error_code: 1,
                        message: "error".to_string(),
                    })
                }
            }
        } else {
            Err(HttpRequestError::BuildError(String::from(
                "Job Detail not matched",
            )))
        }
    }
    async fn parse_response(
        &self,
        response: Response,
        response_type: &String,
        values: &HashMap<String, Vec<Value>>,
    ) -> Result<JobHttpResponseDetail, HttpRequestError> {
        let response_detail = match response_type.as_str() {
            "json" => response
                .text()
                .await
                .map_err(|err| HttpRequestError::SendError(format!("{}", err)))
                .and_then(|content| self.extract_values(content, values)),
            _ => response
                .text()
                .await
                .map_err(|err| HttpRequestError::SendError(format!("{}", err)))
                .and_then(|res| Ok(JobHttpResponseDetail::Body(res))),
        };
        log::debug!("Extracted response detail {:?}", response_detail);
        response_detail
    }
    fn extract_values(
        &self,
        content: String,
        values: &HashMap<String, Vec<Value>>,
    ) -> Result<JobHttpResponseDetail, HttpRequestError> {
        // get result
        let body: Value = serde_json::from_str(&content).map_err(|e| {
            HttpRequestError::GetBodyError(format!("Err {} when parsing response", e))
        })?;

        let mut results = HashMap::<String, Value>::new();
        for (key, paths) in values.iter() {
            let mut ind = 0_usize;
            let mut tmp_value = &body;
            while ind < paths.len() {
                let field: &Value = paths.get(ind).unwrap();
                if field.is_string() && tmp_value.is_object() {
                    tmp_value = &tmp_value[field.as_str().unwrap()]
                } else if field.is_number() && tmp_value.is_array() {
                    tmp_value = &tmp_value[field.as_u64().unwrap() as usize]
                }
                ind = ind + 1;
            }
            results.insert(key.clone(), tmp_value.clone());
        }
        Ok(JobHttpResponseDetail::Values(results))
    }
}

#[async_trait]
impl TaskExecutor for HttpRequestExecutor {
    async fn execute(&self, job: &Job, result_sender: Sender<JobResult>) -> Result<(), Error> {
        debug!("HttpRequestExecutor execute job {:?}", &job);
        let res = self.call_http_request(job).await;
        let response = match res {
            Ok(res) => res,
            Err(err) => err.into(),
        };
        debug!("Http request result {:?}", &response);
        let result = JobHttpResult {
            job: job.clone(),
            //response_timestamp: get_current_time(),
            response,
        };
        let res = result_sender
            .send(JobResult::new(JobResultDetail::HttpRequest(result)))
            .await;
        debug!("send res: {:?}", res);
        Ok(())
    }

    fn can_apply(&self, job: &Job) -> bool {
        match &job.job_detail {
            Some(JobDetail::HttpRequest(_)) => true,
            _ => false,
        }
    }
}
