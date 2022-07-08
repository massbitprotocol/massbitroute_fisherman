use crate::server_builder::SimpleResponse;
use crate::JOB_RESULT_REPORTER_PERIOD;
use anyhow::{anyhow, Error};
use common::jobs::JobResult;
use log::{debug, info, trace};
use serde_json::from_str;
use std::time::{Duration, Instant};
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
                trace!("Received job result: {:?}", job_result);
                results.push(job_result);
            }
            if !results.is_empty() {
                let now = Instant::now();
                info!("Sending {} results.", results.len());
                let res = self.send_results(results).await;
                match res {
                    Ok(_) => {
                        debug!("Success sending results in {:.2?}", now.elapsed());
                    }
                    Err(err) => {
                        info!(
                            "Finished sending results in {:.2?} with Error: {:?}",
                            now.elapsed(),
                            err
                        );
                    }
                }
            } else {
                debug!("No job result for report.");
                sleep(Duration::from_millis(JOB_RESULT_REPORTER_PERIOD)).await;
            }
        }
    }
    pub async fn send_results(&self, results: Vec<JobResult>) -> Result<(), anyhow::Error> {
        let call_back = self.result_callback.to_string();
        info!("Send {} results to: {}", results.len(), call_back);
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
            Ok(res) => {
                if res.status() == 200 {
                    let res = res.text().await?;
                    info!("send_results res:{}", res);
                    let res: SimpleResponse = serde_json::from_str(&*res)?;
                    if res.success == true {
                        Ok(())
                    } else {
                        Err(anyhow!(format!(
                            "send_results response success status: {}",
                            res.success
                        )))
                    }
                } else {
                    Err(anyhow!(format!("res http code: {}", &res.status())))
                }
            }
            Err(err) => Err(anyhow!(format!("{:?}", &err))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;
    use common::logger::init_logger;
    use common::task_spawn;
    use futures::pin_mut;
    use httpmock::prelude::{GET, POST};
    use httpmock::{Mock, MockServer};
    use std::env;
    use tokio::sync::mpsc::{channel, Receiver, Sender};

    const MOCK_WORKER_ID: &str = "7c7da61c-aec7-45b1-9e32-7436d4721ce0";
    const MOCK_REPORT_CALLBACK: &str = "http://127.0.0.1:3031/report";
    const RESULT_NUMBER: usize = 3;
    type Url = String;

    #[tokio::test]
    async fn test_job_result_reporter() -> Result<(), Error> {
        let _res = init_logger(&String::from("Test-fisherman-worker"));

        // Start a lightweight mock server.
        let server = MockServer::start();
        let body = format!("{{ \"success\": \"true\"}}");
        // Create a mock on the server.
        let mock = server.mock(|when, then| {
            when.method(POST).path("/report");
            then.status(200)
                .header("content-type", "text/html; charset=UTF-8")
                .body(body);
        });
        let server_url = server.address().to_string();
        let url = format!("http://{}/report", server_url);

        let (sender, receiver): (Sender<JobResult>, Receiver<JobResult>) = channel(1024);

        tokio::spawn(async move {
            for i in 0..RESULT_NUMBER {
                let job_result = JobResult::default();
                sleep(Duration::from_secs(1)).await;
                if let Err(_) = sender.send(job_result).await {
                    println!("receiver dropped");
                    return;
                }
            }
        });
        info!("url: {}", url);
        let mut reporter = JobResultReporter::new(receiver, url);
        task_spawn::spawn(async move { reporter.run().await });

        sleep(Duration::from_secs(5)).await;

        mock.assert_hits_async(RESULT_NUMBER).await;

        Ok(())
    }
}
