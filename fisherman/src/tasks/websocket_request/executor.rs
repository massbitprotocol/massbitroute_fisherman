use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobDetail, JobResultDetail};
use common::jobs::{Job, JobResult};
use common::tasks::executor::TaskExecutor;
use common::tasks::http_request::{
    HttpRequestError, HttpResponseValues, JobHttpResponse, JobHttpResponseDetail, JobHttpResult,
};
use common::tasks::websocket_request::{JobWebsocketResponse, JobWebsocketResponseDetail};
use common::util::{get_current_time, remove_break_line};
use common::WorkerId;
use log::{debug, error, trace};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use websocket::client::Url;
use websocket::header::Headers;
use websocket::sync::Client;
use websocket::{ClientBuilder, Message, WebSocketResult};

#[derive(Clone, Debug, Default)]
pub struct WebsocketRequestExecutor {
    worker_id: WorkerId,
}

impl WebsocketRequestExecutor {
    pub fn new(worker_id: WorkerId) -> Self {
        WebsocketRequestExecutor { worker_id }
    }
    pub async fn make_request(&self, job: &Job) {}

    // async fn parse_response(
    //     &self,
    //     response: Response,
    //     response_type: &String,
    //     values: &HashMap<String, Vec<Value>>,
    // ) -> Result<JobHttpResponseDetail, HttpRequestError> {
    //     let response_detail = match response_type.as_str() {
    //         "json" => response
    //             .text()
    //             .await
    //             .map_err(|err| HttpRequestError::SendError(format!("{}", err)))
    //             .and_then(|content| self.extract_values(content, values)),
    //         _ => response
    //             .text()
    //             .await
    //             .map_err(|err| HttpRequestError::SendError(format!("{}", err)))
    //             .map(|res| JobHttpResponseDetail::Body(remove_break_line(&res))),
    //     };
    //     log::trace!("Extracted response detail {:?}", response_detail);
    //     response_detail
    // }
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
impl TaskExecutor for WebsocketRequestExecutor {
    async fn execute(&self, job: &Job, result_sender: Sender<JobResult>) -> Result<(), Error> {
        trace!("HttpRequestExecutor execute job {:?}", &job);
        let res = self.make_request(job).await;
        // let response = match res {
        //     Ok(res) => res,
        //     Err(err) => {
        //         JobHttpResponse::new_error(
        //             get_current_time(),
        //             err.get_code(),
        //             err.get_message().as_str(),
        //         )
        //         //err.into()
        //     }
        // };
        // trace!("Http request result {:?}", &response);
        // let result = JobHttpResult {
        //     job: job.clone(),
        //     //response_timestamp: get_current_time(),
        //     response,
        // };
        // if let Some(JobDetail::HttpRequest(request)) = &job.job_detail {
        //     let job_result = JobResult::new(
        //         JobResultDetail::HttpRequest(result),
        //         request.chain_info.clone(),
        //         job,
        //     );
        //     trace!("send job_result: {:?}", job_result);
        //     let res = result_sender.send(job_result).await;
        //     trace!("send res: {:?}", res);
        // };

        Ok(())
    }

    fn can_apply(&self, job: &Job) -> bool {
        match &job.job_detail {
            Some(JobDetail::HttpRequest(_)) => true,
            _ => false,
        }
    }
}
