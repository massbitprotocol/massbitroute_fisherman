use anyhow::Error;
use common::job_manage::{Job, JobDetail, JobPingResult, JobResult};
use common::task_spawn;
use log::info;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

#[async_trait::async_trait]
pub trait JobProcess {
    async fn process(&self) -> Result<JobResult, Error>;

    async fn process_ping(&self) -> Result<JobResult, Error>;
}

#[async_trait::async_trait]
impl JobProcess for Job {
    fn process(&self) -> impl Future<Item = JobResult, Error = Error> {
        let res = task_spawn::spawn(async move {
            let job_detail = self.job_detail.as_ref().unwrap();
            match job_detail {
                JobDetail::Ping(job_detail) => self.process_ping().await,
                JobDetail::Compound(job_detail) => Ok(JobResult::new(self)),
                JobDetail::Benchmark(job_detail) => Ok(JobResult::new(self)),
            }
        });
    }

    async fn process_ping(&self) -> Result<JobResult, Error> {
        for repeat_time in 1..self.repeat_number {
            info!("*** Do ping ***");
            sleep(Duration::from_millis(1000)).await;
        }

        Ok(JobResult::Ping(JobPingResult::default()))
    }
}
