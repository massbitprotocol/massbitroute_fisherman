use crate::models::jobs::JobAssignmentBuffer;
use crate::models::workers::WorkerInfoStorage;
use crate::JOB_DELIVERY_PERIOD;
use common::jobs::{Job, JobAssignment};
use common::workers::Worker;
use common::WorkerId;
use futures_util::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

#[derive(Default)]
pub struct JobDelivery {
    assignment_buffer: Arc<Mutex<JobAssignmentBuffer>>,
    //worker_infos: Arc<WorkerInfoStorage>,
    worker_pool: HashMap<WorkerId, Arc<Worker>>,
}

impl JobDelivery {
    pub fn new(
        //worker_infos: Arc<WorkerInfoStorage>,
        assignment_buffer: Arc<Mutex<JobAssignmentBuffer>>,
    ) -> Self {
        JobDelivery {
            //worker_infos,
            assignment_buffer,
            worker_pool: HashMap::default(),
        }
    }
    pub async fn run(&mut self) {
        loop {
            let assignments = self.assignment_buffer.lock().await.pop_all();
            log::debug!("Run delivery for {} jobs", assignments.len());
            let _undelivered = Vec::<JobAssignment>::default();
            let mut handlers = Vec::new();
            let mut worker_jobs = HashMap::<WorkerId, Vec<Job>>::default();
            for job_assign in assignments.into_iter() {
                let JobAssignment { worker, job, .. } = job_assign;
                let worker_id = worker.get_id();
                if let Some(jobs) = worker_jobs.get_mut(&worker_id) {
                    jobs.push(job)
                } else {
                    worker_jobs.insert(worker_id.clone(), vec![job]);
                }
                self.worker_pool.insert(worker_id, worker);
            }
            for (id, jobs) in worker_jobs.into_iter() {
                if let Some(worker) = self.worker_pool.get(&id) {
                    let worker_cloned = worker.clone();
                    let handler = tokio::spawn(async move {
                        // Process each socket concurrently.
                        worker_cloned.send_jobs(&jobs).await
                    });
                    handlers.push(handler);
                }
            }
            if !handlers.is_empty() {
                join_all(handlers).await;
            }
            sleep(Duration::from_secs(JOB_DELIVERY_PERIOD)).await;
        }
    }
    // fn get_worker(&mut self, worker_id: WorkerId) -> Arc<Worker> {
    //     if let Some(worker) = self.worker_pool.get(&worker_id) {
    //         worker.clone()
    //     } else {
    //         let worker = Arc::new(Worker::new(worker_info.clone()));
    //         self.worker_pool
    //             .insert(worker_info.worker_id.clone(), worker.clone());
    //         worker
    //     }
    // }
}
