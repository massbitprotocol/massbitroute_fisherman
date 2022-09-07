use crate::models::jobs::JobAssignmentBuffer;

use crate::service::submit_chain::ChainAdapter;
use crate::{DELIVERY_PERIOD, IS_REGULAR_WORKER_ONCHAIN};
use common::job_manage::JobRole;
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
        let adapter = ChainAdapter::new();

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

                // Send job
                for (id, jobs) in worker_jobs.into_iter() {
                    if *IS_REGULAR_WORKER_ONCHAIN {
                        let (regular_jobs, verify_jobs) = Self::separate_regular_verify_jobs(jobs);
                        // Decentralize workers
                        let _res = adapter.submit_jobs(&regular_jobs);
                        if let Some(worker) = worker_pool.get(&id) {
                            // Centralize workers
                            let worker_cloned = worker.clone();
                            let handler = tokio::spawn(async move {
                                // Process each socket concurrently.
                                worker_cloned.send_jobs(&verify_jobs).await
                            });
                            handlers.push(handler);
                        }
                        continue;
                    }

                    if let Some(worker) = worker_pool.get(&id) {
                        // Centralize workers
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

    fn separate_regular_verify_jobs(jobs: Vec<Job>) -> (Vec<Job>, Vec<Job>) {
        let mut regular_jobs = vec![];
        let mut verify_jobs = vec![];
        for job in jobs {
            if job.phase == JobRole::Regular {
                regular_jobs.push(job);
            } else {
                verify_jobs.push(job);
            }
        }
        return (regular_jobs, verify_jobs);
    }
}
