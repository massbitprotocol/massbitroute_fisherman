use common::jobs::{AssignmentConfig, Job, JobAssignment};
use common::workers::MatchedWorkers;
use log::{debug, warn};
use rand::Rng;
use std::collections::HashSet;

#[derive(Clone, Debug, Default)]
pub struct JobAssignmentBuffer {
    pub jobs: Vec<Job>,
    pub list_assignments: Vec<JobAssignment>,
}

impl JobAssignmentBuffer {
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
        log::debug!(
            "Assign job {:?} to workers {:?} with config {:?}",
            &job,
            workers,
            assignment_config
        );
        let mut rng = rand::thread_rng();
        match assignment_config {
            None => {
                //without config, assign job for one random nearby worker
                let worker = if !workers.nearby_workers.is_empty() {
                    let ind = rng.gen_range(0..workers.nearby_workers.len());
                    workers.get_nearby_worker(ind)
                } else if !workers.measured_workers.is_empty() {
                    let ind = rng.gen_range(0..workers.measured_workers.len());
                    workers.get_best_worker(ind)
                } else if !workers.remain_workers.is_empty() {
                    workers.get_random_worker()
                } else {
                    warn!("No workers found for component {:?}", job.component_id);
                    None
                };
                if let Some(worker) = worker {
                    debug!(
                        "Assign job {:?}.{:?} to one random worker {:?}",
                        &job.job_name, &job.job_id, &worker.worker_info
                    );
                    let job_assignment = JobAssignment::new(worker, &job);
                    self.list_assignments.push(job_assignment);
                } else {
                    warn!("No workers found for component {:?}", job.component_id);
                }
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
        config: &AssignmentConfig,
    ) {
        debug!(
            "assign_job_with_config {:?} with config {:?} to workers {:?}",
            job, config, workers
        );
        let mut rng = rand::thread_rng();
        if let Some(true) = config.broadcast {
            for worker in workers.measured_workers.iter() {
                let job_assignment = JobAssignment::new(worker.clone(), &job);
                self.list_assignments.push(job_assignment);
                debug!(
                    "Assign job {:?} to worker {:?}",
                    job.job_name,
                    worker.get_url("")
                )
            }
            for worker in workers.remain_workers.iter() {
                let job_assignment = JobAssignment::new(worker.clone(), &job);
                self.list_assignments.push(job_assignment);
                debug!(
                    "Assign job {:?} on provider {:?} to worker {:?}",
                    job.job_name,
                    job.component_id,
                    worker.get_url("")
                )
            }
        } else if let Some(val) = config.worker_number {
            if let Some(true) = config.nearby_only {
                debug!(
                    "assign_job_with_config for job {:?} to {} of workers {:?}",
                    job, val, workers.nearby_workers
                );
                if !workers.nearby_workers.is_empty() {
                    for _i in 0..val {
                        let ind = rng.gen_range(0..workers.nearby_workers.len());
                        let worker = workers.get_nearby_worker(ind).unwrap();
                        let job_assignment = JobAssignment::new(worker.clone(), &job);
                        self.list_assignments.push(job_assignment);
                    }
                }
            } else if let Some(true) = config.by_distance {
                if !workers.measured_workers.is_empty() {
                    for _i in 0..val {
                        let ind = rng.gen_range(0..workers.measured_workers.len());
                        let worker = workers.get_best_worker(ind).unwrap();
                        let job_assignment = JobAssignment::new(worker.clone(), &job);
                        self.list_assignments.push(job_assignment);
                    }
                }
            } else {
                let all_workers = workers.get_all_workers();
                log::debug!(
                    "Try to assign job {:?}.{:?}.{:?} to {} random worker among {} workers",
                    &job.job_type,
                    &job.job_name,
                    &job.job_id,
                    val,
                    all_workers.len()
                );
                if all_workers.len() > 0 {
                    for _ in 0..val {
                        let ind = rng.gen_range(0..all_workers.len());
                        if let Some(worker) = all_workers.get(ind) {
                            debug!(
                                "Assign job {:?} on provider {:?} to worker {:?}",
                                job.job_name,
                                job.component_id,
                                worker.get_url("")
                            );
                            let job_assignment = JobAssignment::new(worker.clone(), &job);
                            self.list_assignments.push(job_assignment);
                        }
                    }
                } else {
                    log::debug!("No worker found in {:?}", &self);
                }
            }
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
    pub fn push_back(&mut self, assignments: Vec<JobAssignment>) {
        for job in assignments.into_iter().rev() {
            self.list_assignments.insert(0, job);
        }
    }
    pub fn pop_all(&mut self) -> Vec<JobAssignment> {
        let mut res = Vec::default();
        res.append(&mut self.list_assignments);
        res
    }
    pub fn get_exist_jobs(&self, plan_id: &str, task_type: &str) -> HashSet<String> {
        self.jobs
            .iter()
            .filter(|job| job.job_type.as_str() == task_type && job.plan_id.as_str() == plan_id)
            .map(|job| job.job_name.clone())
            .collect::<HashSet<String>>()
    }
    pub fn remove_redundant_jobs(self, exist_jobs: &HashSet<String>) -> Self {
        let Self {
            jobs,
            list_assignments,
        } = self;

        let mut active_jobs = Vec::new();
        for job in jobs {
            if !exist_jobs.contains(&job.job_name) {
                active_jobs.push(job);
            }
        }
        let mut active_assignments = Vec::new();
        for assignment in list_assignments {
            if !exist_jobs.contains(&assignment.job.job_name) {
                active_assignments.push(assignment);
            }
        }
        Self {
            jobs: active_jobs,
            list_assignments: active_assignments,
        }
    }
}
