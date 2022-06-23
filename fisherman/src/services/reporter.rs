use crate::JOB_RESULT_REPORTER_PERIOD;
use anyhow::anyhow;
use common::job_manage::JobResultDetail;
use log::debug;
use reqwest::{Client, Error, RequestBuilder, Response};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;

pub struct JobResultReporter {
    receiver: Receiver<JobResultDetail>,
    result_callback: String,
}

impl JobResultReporter {
    pub fn new(receiver: Receiver<JobResultDetail>, result_callback: String) -> Self {
        JobResultReporter {
            receiver,
            result_callback,
        }
    }
    pub async fn run(&mut self) {
        loop {
            let mut results = Vec::<JobResultDetail>::new();
            while let Ok(job_result) = self.receiver.try_recv() {
                debug!("Received job result: {:?}", job_result);
                results.push(job_result);
            }
            if !results.is_empty() {
                debug!("Sending results: {:?}", results);
                self.send_results(results).await;
            } else {
                sleep(Duration::from_millis(JOB_RESULT_REPORTER_PERIOD))
            }
        }
    }
    pub async fn send_results(&self, result: Vec<JobResultDetail>) -> Result<(), anyhow::Error> {
        let call_back = self.result_callback.to_string();
        debug!("Results to send: {:?}, to: {}", result, call_back);
        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        let result = client
            .post(call_back)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&result)?)
            .send()
            .await;
        debug!("Send response: {:?}", result);
        match result {
            Ok(res) => Ok(()),
            Err(err) => Err(anyhow!(format!("{:?}", &err))),
        }
    }
}
