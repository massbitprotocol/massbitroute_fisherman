use crate::models::jobs::{AssignmentBuffer, JobAssignment};
use crate::models::workers::{Worker, WorkerInfoStorage};
use crate::JOB_DELIVERY_PERIOD;
use common::job_manage::Job;
use common::worker::WorkerInfo;
use common::WorkerId;
use futures_util::future::join_all;
use log::{debug, log};
use sea_orm::sea_query::IndexType::Hash;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct JobDelivery {
    assignment_buffer: Arc<Mutex<AssignmentBuffer>>,
    worker_infos: Arc<Mutex<WorkerInfoStorage>>,
    worker_pool: HashMap<WorkerId, Arc<Worker>>,
}

impl JobDelivery {
    pub fn new(
        worker_infos: Arc<Mutex<WorkerInfoStorage>>,
        assignment_buffer: Arc<Mutex<AssignmentBuffer>>,
    ) -> Self {
        JobDelivery {
            worker_infos,
            assignment_buffer,
            worker_pool: HashMap::default(),
        }
    }
    pub async fn run(&mut self) {
        loop {
            let assignments = self.assignment_buffer.lock().await.pop_all();
            log::debug!("Run delivery for {} jobs", assignments.len());
            let undelivered = Vec::<JobAssignment>::default();
            let mut handlers = Vec::new();
            let mut worker_jobs = HashMap::<WorkerId, Vec<Job>>::default();
            let mut workers = HashMap::<WorkerId, Arc<WorkerInfo>>::default();
            for job_assign in assignments.into_iter() {
                let JobAssignment { worker, job, .. } = job_assign;
                if let Some(mut jobs) = worker_jobs.get_mut(&worker.worker_id) {
                    jobs.push(job)
                } else {
                    worker_jobs.insert(worker.worker_id.clone(), vec![job]);
                }
                workers.insert(worker.worker_id.clone(), worker);
            }
            for (id, jobs) in worker_jobs.into_iter() {
                let worker = self.get_worker(workers.get(&id).unwrap().clone());
                let handler = tokio::spawn(async move {
                    // Process each socket concurrently.
                    worker.send_jobs(&jobs).await
                });
                handlers.push(handler);
            }

            join_all(handlers).await;
            sleep(Duration::from_secs(JOB_DELIVERY_PERIOD));
        }
    }
    fn get_worker(&mut self, worker_info: Arc<WorkerInfo>) -> Arc<Worker> {
        if let Some(worker) = self.worker_pool.get(&worker_info.worker_id) {
            worker.clone()
        } else {
            let worker = Arc::new(Worker::new(worker_info.clone()));
            self.worker_pool
                .insert(worker_info.worker_id.clone(), worker.clone());
            worker
        }
    }
}
