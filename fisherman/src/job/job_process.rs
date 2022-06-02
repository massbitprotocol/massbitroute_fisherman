use crate::job::check_module::{CheckComponent, GeneratorBuilder};
use crate::DOMAIN;
use anyhow::Error;
use common::job_manage::{
    Config, Job, JobBenchmark, JobCompound, JobCompoundResult, JobDetail, JobPing, JobPingResult,
    JobResult,
};
use common::task_spawn;
use log::info;
use minifier::js::Keyword::Default;
use std::time::Duration;
use tokio::time::sleep;

#[async_trait::async_trait]
pub trait JobProcess {
    async fn process(self) -> Result<JobResult, Error>;
    async fn process_ping(&self, job_detail: &JobPing) -> Result<JobResult, Error>;
    async fn process_compound(&self, job_detail: &JobCompound) -> Result<JobResult, Error>;
    async fn process_benchmark(&self, job_detail: &JobBenchmark) -> Result<JobResult, Error>;
}

#[async_trait::async_trait]
impl JobProcess for Job {
    async fn process(self) -> Result<JobResult, Error> {
        let res = task_spawn::spawn(async move {
            let job_detail = self.job_detail.as_ref().unwrap();
            match job_detail {
                JobDetail::Ping(job_detail) => self.process_ping(job_detail).await,
                JobDetail::Compound(job_detail) => self.process_compound(job_detail).await,
                JobDetail::Benchmark(job_detail) => self.process_benchmark(job_detail).await,
            }
        })
        .await;
        res?
    }

    async fn process_ping(&self, job_detail: &JobPing) -> Result<JobResult, Error> {
        for repeat_time in 1..self.repeat_number {
            info!("*** Do ping ***");
            sleep(Duration::from_millis(1000)).await;
        }

        Ok(JobResult::Ping(JobPingResult::default()))
    }
    async fn process_compound(&self, job_detail: &JobCompound) -> Result<JobResult, Error> {
        let builder = GeneratorBuilder::default();
        let check_component = builder
            .with_domain((*DOMAIN).to_string())
            .with_base_endpoint(job_detail.base_endpoints.clone())
            .with_config(self.config.clone().unwrap_or_default())
            .build();

        let res = check_component
            .run_check_steps(&job_detail.check_steps, &self.component_info)
            .await;
        println!("run_check_steps: {:?}", res);

        Ok(JobResult::Compound(JobCompoundResult::default()))
    }

    async fn process_benchmark(&self, job_detail: &JobBenchmark) -> Result<JobResult, Error> {
        todo!()
    }
}
