use crate::persistence::seaorm::{
    job_result_benchmarks, job_result_latest_blocks, job_result_pings, jobs, plans, workers,
};
use common::job_manage::{Job, JobBenchmarkResult, JobDetail};
use common::models::PlanEntity;
use common::tasks::eth::JobLatestBlockResult;
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
            plan_id: Set(job.plan_id.to_string()),
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
            plan_id: Set(result.job.plan_id.to_string()),
            worker_id: Set(result.worker_id.to_owned()),
            provider_id: Set(result.job.component_id.to_owned()),
            provider_type: Set(result.job.component_type.to_string()),
            response_time: Set(result.response.response_time as i64),
            ..Default::default()
        }
    }
}

impl From<&JobBenchmarkResult> for job_result_benchmarks::ActiveModel {
    fn from(result: &JobBenchmarkResult) -> Self {
        job_result_benchmarks::ActiveModel {
            job_id: Set(result.job.job_id.to_owned()),
            //job_name: Set(result.job.job_name.to_owned()),
            plan_id: Set(result.job.plan_id.to_string()),
            worker_id: Set(result.worker_id.to_owned()),
            provider_id: Set(result.job.component_id.to_owned()),
            provider_type: Set(result.job.component_type.to_string()),
            request_rate: Set(result.response.request_rate as f64),
            transfer_rate: Set(result.response.transfer_rate as f64),
            average_latency: Set(result.response.average_latency as f64),
            histogram90: Set(result
                .response
                .histograms
                .get(&90)
                .map(|val| val.clone())
                .unwrap_or(0_f32) as f64),
            histogram95: Set(result
                .response
                .histograms
                .get(&95)
                .map(|val| val.clone())
                .unwrap_or(0_f32) as f64),
            histogram99: Set(result
                .response
                .histograms
                .get(&99)
                .map(|val| val.clone())
                .unwrap_or(0_f32) as f64),
            ..Default::default()
        }
    }
}

impl From<&JobLatestBlockResult> for job_result_latest_blocks::ActiveModel {
    fn from(result: &JobLatestBlockResult) -> Self {
        job_result_latest_blocks::ActiveModel {
            job_id: Set(result.job.job_id.to_owned()),
            //job_name: Set(result.job.job_name.to_owned()),
            plan_id: Set(result.job.plan_id.to_string()),
            worker_id: Set(result.worker_id.to_owned()),
            provider_id: Set(result.job.component_id.to_owned()),
            provider_type: Set(result.job.component_type.to_string()),
            block_chain: Set(String::new()),
            block_number: Set(result.response.block_number as i64),
            block_timestamp: Set(result.response.block_timestamp as i64),
            ..Default::default()
        }
    }
}

impl From<&PlanEntity> for plans::ActiveModel {
    fn from(entity: &PlanEntity) -> Self {
        plans::ActiveModel {
            plan_id: Set(entity.plan_id.to_owned()),
            provider_id: Set(entity.provider_id.to_owned()),
            request_time: Set(entity.request_time.to_owned()),
            finish_time: Set(entity.finish_time.to_owned()),
            result: Set(entity.result.to_owned()),
            message: Set(entity.message.to_owned()),
            status: Set(entity.status.to_string()),
            phase: Set(entity.phase.to_owned()),
            ..Default::default()
        }
    }
}
