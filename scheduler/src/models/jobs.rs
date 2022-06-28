use common::component::Zone;
use common::job_manage::JobResultDetail;
use common::jobs::{AssignmentConfig, Job, JobAssignment};
use common::workers::WorkerInfo;
use common::workers::{MatchedWorkers, Worker};
use common::{JobId, WorkerId};
use log::debug;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct AssignmentBuffer {
    pub jobs: Vec<Job>,
    pub list_assignments: Vec<JobAssignment>,
}

impl AssignmentBuffer {
    pub fn new() -> Self {
        Self {
            jobs: vec![],
            list_assignments: vec![],
        }
    }
    pub fn assign_job(
        &mut self,
        job: Job,
        workers: &MatchedWorkers,
        assignment_config: &Option<AssignmentConfig>,
    ) {
        //Do assignment
        let mut rng = rand::thread_rng();
        match assignment_config {
            None => {
                //without config, assign job for one random nearby worker
                let worker = if workers.nearby_workers.len() > 0 {
                    let ind = rng.gen_range(0..workers.nearby_workers.len());
                    workers.nearby_workers.get(ind).unwrap()
                } else {
                    let ind = rng.gen_range(0..workers.best_workers.len());
                    workers.best_workers.get(ind).unwrap()
                };
            }
            Some(config) => {
                self.assign_job_with_config(&job, workers, config);
            }
        }
        self.jobs.push(job);
    }
    fn assign_job_with_config(
        &mut self,
        job: &Job,
        workers: &MatchedWorkers,
        assignment_config: &AssignmentConfig,
    ) {
        for worker in workers.best_workers.iter() {
            let job_assignment = JobAssignment::new(worker.clone(), &job);
            self.list_assignments.push(job_assignment);
            debug!(
                "Assign job {:?} to worker {:?}",
                job.job_name,
                worker.get_url("")
            )
        }
    }
    pub fn append(&mut self, other: Self) {
        let Self {
            mut jobs,
            mut list_assignments,
        } = other;
        self.jobs.append(&mut jobs);
        self.list_assignments.append(&mut list_assignments);
    }
    pub fn add_assignments(&mut self, mut assignments: Vec<JobAssignment>) {
        self.list_assignments.append(&mut assignments);
    }
    pub fn push_back(&mut self, mut assignments: Vec<JobAssignment>) {
        for job in assignments.into_iter().rev() {
            self.list_assignments.insert(0, job);
        }
    }
    pub fn pop_all(&mut self) -> Vec<JobAssignment> {
        let mut res = Vec::default();
        res.append(&mut self.list_assignments);
        res
    }
}
