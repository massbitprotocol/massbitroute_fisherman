use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobDetail, JobResultDetail};
use common::jobs::{Job, JobResult};
use common::tasks::executor::TaskExecutor;
use common::tasks::http_request::{
    HttpRequestError, HttpResponseValues, JobHttpResponse, JobHttpResponseDetail, JobHttpResult,
};
use common::util::{get_current_time, remove_break_line};
use common::WorkerId;
use log::{error, trace};
use reqwest::{Client, Response};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

#[derive(Default)]
pub struct HttpRequestExecutor {
    _worker_id: WorkerId,
    client: Client,
}

impl HttpRequestExecutor {
    pub fn new(worker_id: WorkerId) -> Self {
        HttpRequestExecutor {
            _worker_id: worker_id,
            client: reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap(),
        }
    }
    pub async fn call_http_request(&self, job: &Job) -> Result<JobHttpResponse, HttpRequestError> {
        // Measure response_duration
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
            log::trace!("Request header {:?}", &request.headers);
            //Body
            if let Some(body) = &request.body {
                let req_body = body.clone().to_string();
                log::trace!("Request body {:?}", &req_body);
                req_builder = req_builder.body(req_body);
            }
            //let now = Instant::now();
            let request_time = get_current_time();
            let resp = req_builder
                .send()
                .await
                .map_err(|err| HttpRequestError::SendError(format!("{}", err)))?;
            let http_code = resp.status().as_u16();

            // let response_body = resp
            //     .json()
            //     .await
            //     .map_err(|err| HttpRequestError::GetBodyError(format!("{}", err)))?;

            let response_detail = self
                .parse_response(resp, &request.response_type, &request.response_values)
                .await;
            let response_duration = get_current_time() - request_time;
            match response_detail {
                Ok(detail) => Ok(JobHttpResponse {
                    request_timestamp: request_time,
                    response_duration,
                    detail,
                    http_code,
                    error_code: 0,
                    message: "success".to_string(),
                }),
                Err(err) => {
                    error!("{:?}", &err);
                    Ok(JobHttpResponse {
                        request_timestamp: request_time,
                        response_duration,
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
                .map(|res| JobHttpResponseDetail::Body(remove_break_line(&res))),
        };
        log::trace!("Extracted response detail {:?}", response_detail);
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
        let mut results = HttpResponseValues::default();
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
        trace!("HttpRequestExecutor execute job {:?}", &job);
        let res = self.call_http_request(job).await;
        let response = match res {
            Ok(res) => res,
            Err(err) => {
                JobHttpResponse::new_error(
                    get_current_time(),
                    err.get_code(),
                    err.get_message().as_str(),
                )
                //err.into()
            }
        };
        trace!("Http request result {:?}", &response);
        let result = JobHttpResult {
            job: job.clone(),
            //response_timestamp: get_current_time(),
            response,
        };
        if let Some(JobDetail::HttpRequest(request)) = &job.job_detail {
            let job_result = JobResult::new(
                JobResultDetail::HttpRequest(result),
                request.chain_info.clone(),
                job,
            );
            trace!("send job_result: {:?}", job_result);
            let res = result_sender.send(job_result).await;
            trace!("send res: {:?}", res);
        };

        Ok(())
    }

    fn can_apply(&self, job: &Job) -> bool {
        match &job.job_detail {
            Some(JobDetail::HttpRequest(_)) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::HttpRequestExecutor;
    use anyhow::Error;
    use common::tasks::executor::TaskExecutor;
    use common::tasks::http_request::JobHttpResponseDetail;
    use common::Timestamp;
    use http::response::Builder;
    use httpmock::prelude::GET;
    use httpmock::MockServer;
    use reqwest::{Response, ResponseBuilderExt, Url};
    use std::collections::HashMap;
    use std::ops::Deref;
    use test_util::helper::{mock_job, JobName};

    const MOCK_RTT_RESPONSE_TIME: Timestamp = 123000;

    // fn new_test_job(job_name: &str, component_url: &str) -> Job {
    //     let component = ComponentInfo {
    //         blockchain: "".to_string(),
    //         network: "".to_string(),
    //         id: "".to_string(),
    //         user_id: "".to_string(),
    //         ip: "".to_string(),
    //         zone: Default::default(),
    //         country_code: "".to_string(),
    //         token: "".to_string(),
    //         component_type: Default::default(),
    //         endpoint: None,
    //         status: "".to_string(),
    //     };
    //     let job_detail = r###"
    //     {"url": "", "body": "", "method": "get", "headers": {}, "chain_info": {"chain": "eth", "network": "mainnet"}, "response_type": "text", "response_values": {}}
    //     "###;
    //     let mut job_http_request: JobHttpRequest = serde_json::from_str(job_detail).unwrap();
    //     job_http_request.url = component_url.to_string();
    //     let mut job = match job_name {
    //         "benchmark" => Job::new(
    //             "benchmark".to_string(),
    //             "".to_string(),
    //             "".to_string(),
    //             &component,
    //             JobDetail::Benchmark(JobBenchmark::default()),
    //             Default::default(),
    //         ),
    //         "http" => Job::new(
    //             "http".to_string(),
    //             "HttpRequest".to_string(),
    //             "".to_string(),
    //             &component,
    //             JobDetail::HttpRequest(job_http_request),
    //             Default::default(),
    //         ),
    //         _ => Job::default(),
    //     };
    //     job.component_url = component_url.to_string();
    //     job
    // }

    fn new_mock_eth_block_response() -> Response {
        let url = Url::parse("http://example.com").unwrap();
        let body = r###"
{
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
        "baseFeePerGas": "0x919d4f7bb",
        "difficulty": "0x32d4839f833850",
        "extraData": "0x36",
        "gasLimit": "0x1c8dea2",
        "gasUsed": "0x183ea29",
        "hash": "0x6a4012e041f5fa58855dde864b6371c03d4d31d17b993dc8d947d9fdac65b1d2",
        "logsBloom": "0x6ffadefff1855efbd9dd3bf6afbeabe7556aee9df9773cf44346fe1bf7fead6375f29771fe2ffaea6ab779723fef1da47badf9d31fdffbf3b674acb7327ff7a6ddfcfbaf8eb89fecfccfecdba3a26a773eefae7f356796ddcaf1fe9591afb756ff8bfad80ebae7f7d37f07c3acb1febef5bf7fdcee5d8fc55b794fffcfcdeab2b5b5e79efdf4fb8fb1ae6609ed9e7ed79c37ffe1e5def7ddd9ad3bf5e39e55e63a7df33d3b2fbc15feadc7b65ff7e6f8feff6e3fcfe6bee78da9d6fe96afffebdf3ef6a2e3b3e1f55fb17dbb5f9bdd66da7db44fba7ce55daf8d7d6fe27ee4fbbebbf66affd76c998f0317fa7fa4eabcbe3367cdf7b0f5de9f5ffca2f3b87de2",
        "miner": "0x4069e799da927c06b430e247b2ee16c03e8b837d",
        "mixHash": "0x9bf528940b7de0c4cb765d8a714f4c5ed132dad25b68033a86ee3c7efbd47c9f",
        "nonce": "0xf35487b463a7b3d7",
        "number": "0xe2e63a",
        "parentHash": "0x969fe278cbed922ebcc9b0046d12f389ab33fbd4edffb55901ade2c5b21118dc",
        "receiptsRoot": "0x82c93301a2878062d2a3a9a862fd77967edbaa24e79597c5965dafa2b66350f4",
        "sha3Uncles": "0x3dbbb28a8f929baf490fea113c5c0f9077c6b1b04b71eb9e1160b5c14167cfc5",
        "size": "0x1732f",
        "stateRoot": "0x2934c182c64080672c6df30f542afaca4cd6d2b03876d65d6b80d88b4576116b",
        "timestamp": "0x62942e59",
        "totalDifficulty": "0xaa8bfea8ded3c5c4219",
        "transactions": [
            "0x4e3a9d6d13b12752c5e0ef21a08565557f8a4f50d171375fca8fe8e260bf1ffe",
            "0x00e02ace23596690cba908ef0d902db549da3ede0c32e54d251a16b55434983f"
        ],
        "transactionsRoot": "0x3d056a51333a14eef1b6a9fef543b3aa34f216c5c47d841e223d7cbf2af17293",
        "uncles": [
            "0xb5a53e43f77611120c9a2513e375fb47cff2eaaddf0364c65e27b7312654fdd0"
        ]
    }
}
        "###;
        let response = Builder::new()
            .status(200)
            .url(url.clone())
            .body(body)
            .unwrap();
        let response = Response::from(response);
        response
    }

    fn run_mock_provider_server() -> MockServer {
        // Start a lightweight mock server.
        let server = MockServer::start();

        // Create a mock on the server.
        let hello_mock = server.mock(|when, then| {
            when.method(GET).path("/_rtt");
            then.status(200)
                .header("content-type", "text/html; charset=UTF-8")
                .body(format!("{}\n", MOCK_RTT_RESPONSE_TIME));
        });
        server
    }

    fn new_mock_rtt_response(rtt: i64) -> Response {
        let url = Url::parse("http://example.com").unwrap();
        let body = format!("{}\n", rtt);
        let response = Builder::new()
            .status(200)
            .url(url.clone())
            .body(body)
            .unwrap();
        let response = Response::from(response);
        response
    }

    fn new_executor() -> HttpRequestExecutor {
        HttpRequestExecutor::new("test_worker_id".to_string())
    }

    #[test]
    fn test_can_apply() {
        let executor = new_executor();
        let job_benchmark = mock_job(&JobName::Benchmark, "", "", &Default::default());
        let job_http = mock_job(&JobName::RoundTripTime, "", "", &Default::default());

        assert_eq!(executor.can_apply(&job_benchmark), false);
        assert_eq!(executor.can_apply(&job_http), true);
    }

    #[tokio::test]
    async fn test_parse_response_latest_block() -> Result<(), Error> {
        let response = new_mock_eth_block_response();
        let executor = new_executor();
        let values = serde_json::from_value(serde_json::json!({
          "hash": ["result", "hash"],
          "number": ["result", "number"],
          "timestamp": ["result", "timestamp"]
        }))?;
        let res = executor
            .parse_response(response, &"json".to_string(), &values)
            .await?;
        let expect_res = HashMap::from([
            (
                "hash".to_string(),
                serde_json::to_value(
                    "0x6a4012e041f5fa58855dde864b6371c03d4d31d17b993dc8d947d9fdac65b1d2",
                )
                .unwrap(),
            ),
            (
                "number".to_string(),
                serde_json::to_value("0xe2e63a").unwrap(),
            ),
            (
                "timestamp".to_string(),
                serde_json::to_value("0x62942e59").unwrap(),
            ),
        ]);

        if let JobHttpResponseDetail::Values(res) = res {
            assert_eq!(res.deref(), &expect_res);
        } else {
            panic!("False parse_response");
        }
        let rtt = 123000i64;
        let response = new_mock_rtt_response(rtt);
        let res = executor
            .parse_response(response, &"text".to_string(), &values)
            .await?;

        if let JobHttpResponseDetail::Body(res) = res {
            assert_eq!(res.parse::<i64>().unwrap(), rtt);
        } else {
            panic!("False parse_response");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_response_rtt() -> Result<(), Error> {
        let executor = new_executor();
        let rtt = 123000i64;
        let response = new_mock_rtt_response(rtt);
        let res = executor
            .parse_response(response, &"text".to_string(), &Default::default())
            .await?;

        if let JobHttpResponseDetail::Body(res) = res {
            assert_eq!(res.parse::<i64>().unwrap(), rtt);
        } else {
            panic!("False parse_response");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_call_http_request() -> Result<(), Error> {
        let executor = new_executor();

        // Create mock provider server
        let mock_provider = run_mock_provider_server();
        let url = format!("http://{}/_rtt", mock_provider.address());
        println!("url: {}", url);

        // Create Job that point to provider mock server
        let job_http = mock_job(
            &JobName::RoundTripTime,
            url.as_str(),
            "",
            &Default::default(),
        );
        println!("job: {:?}", job_http);

        let res = executor.call_http_request(&job_http).await;

        // Check result
        if let Ok(JobHttpResponse {
            detail: JobHttpResponseDetail::Body(body),
            ..
        }) = res
        {
            assert_eq!(body.parse::<Timestamp>()?, MOCK_RTT_RESPONSE_TIME);
        } else {
            panic!("Wrong call_http_request res");
        }

        Ok(())
    }
}
