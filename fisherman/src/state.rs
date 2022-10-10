use crate::models::job::JobBuffer;
use common::jobs::Job;
use common::{JobId, PlanId};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Default)]
pub struct WorkerState {
    pub job_buffer: Arc<Mutex<JobBuffer>>,
}

impl WorkerState {
    pub fn new(job_buffer: Arc<Mutex<JobBuffer>>) -> Self {
        WorkerState { job_buffer }
    }
    pub async fn push_jobs(&mut self, jobs: Vec<Job>) -> usize {
        {
            let mut lock = self.job_buffer.lock().await;
            lock.add_jobs(jobs)
        }
    }
    pub async fn cancel_jobs(&mut self, jobs: Vec<JobId>) -> usize {
        {
            let mut lock = self.job_buffer.lock().await;
            lock.cancel_jobs(jobs)
        }
    }
    pub async fn cancel_plans(&mut self, plans: Vec<PlanId>) -> usize {
        {
            let mut lock = self.job_buffer.lock().await;
            lock.cancel_plans(plans)
        }
    }
    pub async fn queue_len(&mut self) -> usize {
        {
            self.job_buffer.lock().await.len()
        }
    }
}
