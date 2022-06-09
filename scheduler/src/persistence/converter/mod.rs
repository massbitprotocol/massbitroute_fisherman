use crate::persistence::seaorm::{job_result_pings, jobs, workers};
use common::job_manage::{Job, JobDetail};
use common::tasks::ping::JobPingResult;
use common::worker::WorkerInfo;
use core::default::Default;
use log::debug;
use sea_orm::ActiveValue::Set;
use sea_orm::NotSet;
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
        let header = match job.header.is_empty() {
            true => NotSet,
            false => match serde_json::to_value(job.header.to_owned()) {
                Ok(value) => Set(Some(value)),
                Err(err) => {
                    debug!("err from job_detail: {:?}", err);
                    NotSet
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
            component_url: Set(job.component_url.to_owned()),
            header,
            interval: Set(job.interval),
            repeat_number: Set(job.repeat_number),
            id: NotSet,
        }
    }
}

impl From<&JobPingResult> for job_result_pings::ActiveModel {
    fn from(result: &JobPingResult) -> Self {
        job_result_pings::ActiveModel {
            job_id: Set(result.job.job_id.to_owned()),
            //job_name: Set(result.job.job_name.to_owned()),
            worker_id: Set(result.re.component_id.to_owned()),
            provider_id: Set(result.job.component_id.to_owned()),
            priority: Set(job.priority),
            expected_runtime: Set(job.expected_runtime as i64),
            parallelable: Set(job.parallelable),
            timeout: Set(job.timeout as i64),
            job_detail: Set(job_detail),
            ..Default::default()
        }
    }
}
