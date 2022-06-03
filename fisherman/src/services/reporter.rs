use crate::JOB_RESULT_REPORTER_PERIOD;
use common::job_manage::JobResult;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;

pub struct JobResultReporter {
    receiver: Receiver<JobResult>,
}

impl JobResultReporter {
    pub fn new(receiver: Receiver<JobResult>) -> Self {
        JobResultReporter { receiver }
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
    pub async fn send_results(&self, result: Vec<JobResult>) -> Result<(), anyhow::Error> {}
}
