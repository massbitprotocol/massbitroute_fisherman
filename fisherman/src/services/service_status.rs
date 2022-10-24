use crate::models::job::JobBuffer;
use common::jobs::JobResult;
use common::workers::WorkerStatus;
use common::COMMON_CONFIG;
use log::debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;

pub struct WorkerStatusCheck {
    worker_status: Arc<RwLock<WorkerStatus>>,
    sender: Sender<JobResult>,
    job_buffer: Arc<Mutex<JobBuffer>>,
}

impl WorkerStatusCheck {
    pub fn new(sender: Sender<JobResult>, job_buffer: Arc<Mutex<JobBuffer>>) -> Self {
        WorkerStatusCheck {
            worker_status: Arc::new(RwLock::new(WorkerStatus::default())),
            sender,
            job_buffer,
        }
    }
}

impl WorkerStatusCheck {
    pub async fn run(self) {
        loop {
            // Update status
            self.update_status().await;
            sleep(Duration::from_millis(COMMON_CONFIG.update_status_interval)).await;
        }
    }
    async fn update_status(&self) {
        // Update status
        let job_buffers = self.job_buffer.lock().await;
        let jobs_number_in_queue = job_buffers.len();
        let jobs_stat = job_buffers.statistic_jobs();
        let reports_number_in_queue = self.sender.max_capacity() - self.sender.capacity();
        let worker_status = WorkerStatus {
            jobs_number_in_queue,
            reports_number_in_queue,
            jobs_stat,
        };
        debug!("Worker status: {:?}", &worker_status);
        *self.worker_status.write().await = worker_status;
    }
    pub fn get_status(&self) -> Arc<RwLock<WorkerStatus>> {
        // Update status
        self.worker_status.clone()
    }
}
