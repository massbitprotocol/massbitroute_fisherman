use common::jobs::Job;
use common::util::get_current_time;
use common::Timestamp;
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct JobBuffer {
    jobs: VecDeque<Job>,
}

impl JobBuffer {
    pub fn new() -> Self {
        JobBuffer {
            jobs: VecDeque::new(),
        }
    }
    pub fn add_job(&mut self, job: Job) {
        let mut next_ind = self.jobs.len();
        for (ind, item) in self.jobs.iter().enumerate() {
            if job.expected_runtime > 0 {
                if job.expected_runtime < item.expected_runtime {
                    next_ind = ind;
                    break;
                }
            } else if item.expected_runtime == 0 && job.priority < item.priority {
                next_ind = ind;
                break;
            }
        }
        log::trace!("insert job {:?} to index of queue {}", &job, next_ind);
        self.jobs.insert(next_ind, job);
    }
    pub fn get_job_existed_position(&mut self, job: &Job) -> Option<usize> {
        self.jobs
            .iter()
            .position(|job_in_queue| job_in_queue.is_eq(job))
    }

    pub fn add_jobs(&mut self, mut jobs: Vec<Job>) -> usize {
        for job in jobs {
            let pos = self.get_job_existed_position(&job);
            if let Some(pos) = pos {
                self.jobs.remove(pos);
            };

            //Find index for new job
            let mut next_ind = self.jobs.len();
            for (ind, item) in self.jobs.iter().enumerate() {
                if job.expected_runtime > 0 {
                    if job.expected_runtime < item.expected_runtime {
                        next_ind = ind;
                        break;
                    }
                } else if item.expected_runtime == 0 && job.priority < item.priority {
                    next_ind = ind;
                    break;
                }
            }
            self.jobs.insert(next_ind, job);
        }
        self.jobs.len()
    }
    pub fn pop_job(&mut self) -> Option<Job> {
        let first_expected_time = self.jobs.front().and_then(|job| {
            log::trace!(
                "Found new job with expected runtime {}: {:?}",
                &job.expected_runtime,
                job
            );
            Some(job.expected_runtime)
        });
        if let Some(expected_time) = first_expected_time {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .expect("Unix time doesn't go backwards;")
                .as_millis() as Timestamp;
            if expected_time <= current_time {
                trace!(
                    "Found job is executed after {}. Job with runtime {}. Current time: {}",
                    expected_time - current_time,
                    expected_time,
                    current_time
                );
                let job = self.jobs.pop_front();
                if let Some(inner) = job.as_ref() {
                    let mut next_job = inner.clone();
                    if inner.repeat_number > 0 {
                        next_job.expected_runtime = current_time + inner.interval;
                        next_job.repeat_number = next_job.repeat_number - 1;
                        trace!("Schedule new repeat job: {:?}", &next_job);
                        self.add_job(next_job);
                    }
                }
                job
            } else {
                None
            }
        } else {
            //log::trace!("Job queue is empty");
            None
        }
    }
}
