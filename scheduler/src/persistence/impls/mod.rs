use crate::persistence::seaorm::{jobs, workers};
use common::job_manage::Job;
use common::worker::WorkerInfo;
use core::default::Default;
use sea_orm::ActiveValue::Set;

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
        jobs::ActiveModel {
            job_id: Set(job.job_id.to_owned()),
            component_id: Set(job.component_id.to_owned()),
            priority: Set(job.priority),
            expected_runtime: Set(job.expected_runtime as i64),
            parallelable: Set(job.parallelable),
            timeout: Set(job.timeout as i64),
            //job_detail: Set(job
            //    .job_detail
            //    .map(|detail| serde_json::to_value(detail).ok())),
            ..Default::default()
        }
    }
}
