use crate::models::job::JobBuffer;
use crate::JOB_EXECUTOR_PERIOD;
use common::job_manage::{Job, JobResult};
use common::tasks::executor::TaskExecutor;
use common::tasks::get_eth_executors;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

pub struct JobExecution {
    sender: Sender<JobResult>,
    job_buffers: Arc<Mutex<JobBuffer>>,
    executors: Vec<Arc<dyn TaskExecutor>>,
}

impl JobExecution {
    pub fn new(sender: Sender<JobResult>, job_buffers: Arc<Mutex<JobBuffer>>) -> Self {
        let executors = get_eth_executors();
        JobExecution {
            sender,
            job_buffers,
            executors,
        }
    }
    pub async fn run(&mut self) {
        loop {
            let next_job = self.job_buffers.lock().await.pop_job();
            if let Some(job) = next_job {
                for executor in self.executors.iter() {
                    let sender = self.sender.clone();
                    executor.execute(&job, sender).await;
                }
            }

            sleep(Duration::from_secs(JOB_EXECUTOR_PERIOD))
        }
    }
}
