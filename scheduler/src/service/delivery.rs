use crate::models::jobs::JobAssignmentBuffer;

use crate::DELIVERY_PERIOD;
use common::jobs::{Job, JobAssignment};
use common::workers::Worker;
use common::{PlanId, WorkerId};
use futures_util::future::{join, join_all};
use log::error;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::sleep;

#[derive(Default)]
pub struct JobDelivery {
    assignment_buffer: Arc<Mutex<JobAssignmentBuffer>>,
    cancel_plans_buffer: Arc<Mutex<CancelPlanBuffer>>,
}

#[derive(Default, Clone)]
pub struct CancelPlanBuffer {
    inner: HashMap<Arc<Worker>, Vec<PlanId>>,
}

impl CancelPlanBuffer {
    fn pop_all(&mut self) -> HashMap<Arc<Worker>, Vec<PlanId>> {
        let res = self.inner.clone();
        self.inner = HashMap::new();
        res
    }
    pub fn insert_plan(&mut self, plan: PlanId, worker: Arc<Worker>) {
        let entry = self.inner.entry(worker).or_insert(vec![]);
        entry.push(plan);
    }
}

impl Deref for CancelPlanBuffer {
    type Target = HashMap<Arc<Worker>, Vec<PlanId>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CancelPlanBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl JobDelivery {
    pub fn new(
        assignment_buffer: Arc<Mutex<JobAssignmentBuffer>>,
        cancel_plans_buffer: Arc<Mutex<CancelPlanBuffer>>,
    ) -> Self {
        JobDelivery {
            //worker_infos,
            assignment_buffer,

            cancel_plans_buffer,
        }
    }
    pub async fn run(&self) {
        let cancel_plans_buffer = self.cancel_plans_buffer.clone();
        let assignment_buffer = self.assignment_buffer.clone();
        let task_assignment_buffer = task::spawn(async move {
            let mut worker_pool: HashMap<WorkerId, Arc<Worker>> = HashMap::default();
            loop {
                let assignments = assignment_buffer.lock().await.pop_all();
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
                    worker_pool.insert(worker_id, worker);
                }
                for (id, jobs) in worker_jobs.into_iter() {
                    if let Some(worker) = worker_pool.get(&id) {
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
                sleep(Duration::from_secs(DELIVERY_PERIOD)).await;
            }
        });

        let task_cancel_plans_buffer = task::spawn(async move {
            loop {
                let cancel_plans = cancel_plans_buffer.lock().await.pop_all();
                for (worker, plans) in cancel_plans.into_iter() {
                    let res = worker.send_cancel_plans(&plans).await;
                    if let Err(err) = res {
                        error!("send_cancel_plans error: {:?}", err);
                    }
                }
                sleep(Duration::from_secs(DELIVERY_PERIOD)).await;
            }
        });

        let res = join(task_assignment_buffer, task_cancel_plans_buffer).await;
        error!("JobDelivery stop with error: {:?}", res);
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
