use common::job_manage::Job;
use common::util::get_current_time;
use common::Timestamp;
use log::{debug, info};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct JobBuffer {
    jobs: Vec<Job>,
}

impl JobBuffer {
    pub fn new() -> Self {
        JobBuffer { jobs: vec![] }
    }
    pub fn add_job(&mut self, job: Job) {
        self.jobs.push(job);
    }
    pub fn add_jobs(&mut self, mut jobs: Vec<Job>) {
        for job in jobs {
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
    }
    pub fn pop_job(&mut self) -> Option<Job> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("Unix time doesn't go backwards; qed")
            .as_millis() as Timestamp;
        let first_expected_time = self
            .jobs
            .first()
            .and_then(|job| Some(job.expected_runtime))
            .unwrap_or(0_128);
        if first_expected_time < current_time {
            let job = self.jobs.pop();
            if let Some(inner) = job.as_ref() {
                let mut next_job = inner.clone();
                if inner.repeat_number > 0 {
                    next_job.expected_runtime = get_current_time() + inner.interval;
                    next_job.repeat_number = next_job.repeat_number - 1;
                    debug!("Schedule new repeat job: {:?}", next_job);
                }
            }
            job
        } else {
            None
        }
    }
}
