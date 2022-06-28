use common::component::Zone;
use common::job_manage::JobResultDetail;
use common::jobs::{Job, JobAssignment};
use common::workers::WorkerInfo;
use common::workers::{MatchedWorkers, Worker};
use common::{JobId, WorkerId};
use log::debug;
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
    pub fn assign_job(&mut self, job: Job, workers: &MatchedWorkers) {
        //Do assignment
        for worker in workers.best_workers.iter() {
            let job_assignment = JobAssignment::new(worker.clone(), &job);
            self.list_assignments.push(job_assignment);
            debug!(
                "Assign job {:?} to worker {:?}",
                job.job_name,
                worker.get_url("")
            )
        }
        self.jobs.push(job);
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
