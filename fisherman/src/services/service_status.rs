use crate::models::job::JobBuffer;
use common::jobs::JobResult;
use common::workers::WorkerStatus;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;

const UPDATE_STATUS_INTERVAL: u64 = 1000;

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
            sleep(Duration::from_micros(UPDATE_STATUS_INTERVAL)).await;
        }
    }
    async fn update_status(&self) {
        // Update status
        let jobs_number_in_queue = self.job_buffer.lock().await.len();
        let reports_number_in_queue = self.sender.max_capacity() - self.sender.capacity();
        *self.worker_status.write().await = WorkerStatus {
            jobs_number_in_queue,
            reports_number_in_queue,
        };
    }
    pub fn get_status(&self) -> Arc<RwLock<WorkerStatus>> {
        // Update status
        self.worker_status.clone()
    }
}
