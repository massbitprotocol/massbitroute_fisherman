use crate::JOB_RESULT_REPORTER_PERIOD;
use anyhow::anyhow;
use common::jobs::JobResult;
use log::{debug, info, trace};
//use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::time::sleep;

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
            while let Ok(job_result) = self.receiver.try_recv() {
                info!("Received job result: {:?}", job_result);
                results.push(job_result);
            }
            if !results.is_empty() {
                info!("Sending {} results.", results.len());
                self.send_results(results).await;
                info!("Finished sending results.");
            } else {
                debug!("No job result for report.");
                sleep(Duration::from_millis(JOB_RESULT_REPORTER_PERIOD)).await;
            }
        }
    }
    pub async fn send_results(&self, results: Vec<JobResult>) -> Result<(), anyhow::Error> {
        let call_back = self.result_callback.to_string();
        trace!("Send {} results to: {}", results.len(), call_back);
        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        let body = serde_json::to_string(&results)?;
        info!("sending body len: {}", body.len());
        let result = client
            .post(call_back)
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await;
        trace!("Send response: {:?}", result);
        match result {
            Ok(res) => Ok(()),
            Err(err) => Err(anyhow!(format!("{:?}", &err))),
        }
    }
}
