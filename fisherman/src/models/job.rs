use common::jobs::Job;
use common::util::get_current_time;

use log::trace;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

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
    /// Add job by expected_runtime and priority order
    fn add_job(&mut self, job: Job) {
        // Check if it is duplicate job and remove the duplicate old job
        let pos = self.get_job_existed_position(&job);
        if let Some(pos) = pos {
            self.jobs.remove(pos);
        };
        // Find position for insert job
        let mut next_ind = self.jobs.len();
        for (ind, item) in self.jobs.iter().enumerate() {
            if job.expected_runtime < item.expected_runtime
                || (job.expected_runtime == item.expected_runtime && job.priority < item.priority)
            {
                next_ind = ind;
                break;
            }
        }
        log::trace!("insert job {:?} to index of queue {}", &job, next_ind);
        self.jobs.insert(next_ind, job);
    }
    /// Add job by expected_runtime and priority order
    pub fn get_job_existed_position(&mut self, job: &Job) -> Option<usize> {
        self.jobs
            .iter()
            .position(|job_in_queue| job_in_queue.is_eq(job))
    }

    pub fn add_jobs(&mut self, jobs: Vec<Job>) -> usize {
        for job in jobs {
            self.add_job(job);
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
            let current_time = get_current_time();
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

#[cfg(test)]
mod test {
    use crate::models::job::JobBuffer;
    
    use common::job_manage::JobDetail;
    use common::jobs::Job;
    use common::tasks::http_request::JobHttpRequest;
    use common::util::get_current_time;
    use common::Timestamp;

    fn new_test_job(expected_runtime: Timestamp, priority: i32, plan_id: String) -> Job {
        let mut job = Job::new(
            plan_id.clone(),
            plan_id,
            "".to_string(),
            &Default::default(),
            JobDetail::HttpRequest(JobHttpRequest::default()),
            Default::default(),
        );
        job.priority = priority;
        job.expected_runtime = expected_runtime;
        job
    }

    #[test]
    fn test_add_job() {
        let now = get_current_time();
        let mut buffer = JobBuffer::new();
        let job_1 = new_test_job(0, 1, "job_1".to_string());
        let job_2 = new_test_job(0, 2, "job_2".to_string());
        let job_3 = new_test_job(now, 1, "job_3".to_string());
        let job_4 = new_test_job(now + 1000, 1, "job_4".to_string());
        let job_5 = new_test_job(now + 2000, 1, "job_5".to_string());
        buffer.add_job(job_4);
        buffer.add_job(job_2);
        buffer.add_job(job_3);
        buffer.add_job(job_1);
        buffer.add_job(job_5);
        for job in buffer.jobs.iter() {
            println!("job: {}", job.plan_id);
        }
        assert_eq!(buffer.jobs.pop_front().unwrap().plan_id, "job_1");
        assert_eq!(buffer.jobs.pop_front().unwrap().plan_id, "job_2");
        assert_eq!(buffer.jobs.pop_front().unwrap().plan_id, "job_3");
        assert_eq!(buffer.jobs.pop_front().unwrap().plan_id, "job_4");
        assert_eq!(buffer.jobs.pop_front().unwrap().plan_id, "job_5");
    }

    #[test]
    fn test_add_jobs() {
        let now = get_current_time();
        let mut buffer = JobBuffer::new();
        let job_1 = new_test_job(0, 1, "job_1".to_string());
        let job_2 = new_test_job(0, 2, "job_2".to_string());
        let job_3 = new_test_job(now, 1, "job_3".to_string());
        let job_4 = new_test_job(now + 1000, 1, "job_4".to_string());
        let job_5 = new_test_job(now + 2000, 1, "job_5".to_string());
        buffer.add_job(job_4);
        buffer.add_job(job_2);

        buffer.add_jobs(vec![job_3, job_1, job_5]);

        for job in buffer.jobs.iter() {
            println!("job: {}", job.plan_id);
        }
        assert_eq!(buffer.jobs.pop_front().unwrap().plan_id, "job_1");
        assert_eq!(buffer.jobs.pop_front().unwrap().plan_id, "job_2");
        assert_eq!(buffer.jobs.pop_front().unwrap().plan_id, "job_3");
        assert_eq!(buffer.jobs.pop_front().unwrap().plan_id, "job_4");
        assert_eq!(buffer.jobs.pop_front().unwrap().plan_id, "job_5");
    }

    #[test]
    fn test_add_same_job() {
        let now = get_current_time();
        let mut buffer = JobBuffer::new();
        let job_1 = new_test_job(0, 1, "job_1".to_string());
        let job_2 = new_test_job(0, 2, "job_2".to_string());
        let job_3 = new_test_job(now, 1, "job_3".to_string());
        let job_4 = new_test_job(now + 1000, 1, "job_4".to_string());
        let job_5 = new_test_job(now + 2000, 1, "job_5".to_string());
        let same_job_5 = new_test_job(now + 2000, 1, "job_5".to_string());
        buffer.add_job(job_1);
        buffer.add_job(job_2);
        buffer.add_job(job_3);
        buffer.add_job(job_4);
        buffer.add_job(job_5);
        buffer.add_job(same_job_5);
        for job in buffer.jobs.iter() {
            println!("job: {}", job.plan_id);
        }
        assert_eq!(buffer.jobs.pop_front().unwrap().plan_id, "job_1");
        assert_eq!(buffer.jobs.pop_front().unwrap().plan_id, "job_2");
        assert_eq!(buffer.jobs.pop_front().unwrap().plan_id, "job_3");
        assert_eq!(buffer.jobs.pop_front().unwrap().plan_id, "job_4");
        assert_eq!(buffer.jobs.pop_front().unwrap().plan_id, "job_5");
    }
}
