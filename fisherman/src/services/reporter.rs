use crate::JOB_RESULT_REPORTER_PERIOD;
use anyhow::anyhow;
use common::job_manage::JobResult;
use reqwest::{Client, Error, RequestBuilder, Response};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;

pub struct JobResultReporter {
    receiver: Receiver<JobResult>,
    result_callback: String,
}

impl JobResultReporter {
    pub fn new(receiver: Receiver<JobResult>, result_callback: String) -> Self {
        JobResultReporter {
            receiver,
            result_callback,
        }
    }
    pub async fn run(&mut self) {
        loop {
            let mut results = Vec::<JobResult>::new();
            while let Some(job_result) = self.receiver.recv().await {
                results.push(job_result);
            }
            self.send_results(results).await;
            sleep(Duration::from_secs(JOB_RESULT_REPORTER_PERIOD))
        }
    }
    pub async fn send_results(&self, result: Vec<JobResult>) -> Result<(), anyhow::Error> {
        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        let result = client
            .clone()
            .post(self.result_callback.to_string())
            .header("content-type", "application/json")
            .body(serde_json::to_string(&result)?)
            .send()
            .await;
        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow!(format!("{:?}", &err))),
        }
    }
}
