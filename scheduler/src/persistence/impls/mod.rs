use crate::persistence::seaorm::{jobs, workers};
use common::job_manage::{Job, JobDetail};
use common::worker::WorkerInfo;
use core::default::Default;
use log::debug;
use sea_orm::ActiveValue::Set;
use serde_json::{Error, Value};

impl From<&WorkerInfo> for workers::ActiveModel {
    fn from(worker: &WorkerInfo) -> Self {
        let workers = workers::ActiveModel {
            worker_id: Set(worker.worker_id.to_owned()),
            worker_ip: Set(worker.worker_ip.to_owned()),
            active: Set(1),
            zone: Set(format!("{:?}", &worker.zone)),
            url: Set(worker.url.to_owned()),
            ..Default::default()
        };
        workers
    }
}

impl From<&Job> for jobs::ActiveModel {
    fn from(job: &Job) -> Self {
        debug!("job.job_detail: {:?}", job.job_detail);
        let job_detail = match &job.job_detail {
            None => None,
            Some(detail) => match serde_json::to_value(detail) {
                Ok(value) => Some(value),
                Err(err) => {
                    debug!("err from job_detail: {:?}", err);
                    None
                }
            },
        };
        debug!("from job_detail: {:?}", job_detail);

        jobs::ActiveModel {
            job_id: Set(job.job_id.to_owned()),
            job_name: Set(job.job_name.to_owned()),
            component_id: Set(job.component_id.to_owned()),
            priority: Set(job.priority),
            expected_runtime: Set(job.expected_runtime as i64),
            parallelable: Set(job.parallelable),
            timeout: Set(job.timeout as i64),
            job_detail: Set(job_detail),
            ..Default::default()
        }
    }
}
